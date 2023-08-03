pub mod states;

use std::sync::Arc;

use aries_vcx_core::ledger::base_ledger::IndyLedgerRead;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use chrono::Utc;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::{
        connection::{
            invitation::InvitationContent,
            request::{Request, RequestContent, RequestDecorators},
            response::Response,
            ConnectionData,
        },
        notification::ack::{Ack, AckContent, AckDecorators, AckStatus},
    },
};
use url::Url;
use uuid::Uuid;

use crate::{
    common::ledger::transactions::into_did_doc,
    errors::error::VcxResult,
    handlers::util::{matches_thread_id, AnyInvitation},
    protocols::connection::trait_bounds::ThreadId,
    transport::Transport,
};

use self::states::{
    completed::Completed, initial::Initial, invited::Invited, requested::Requested, responded::Responded,
};

use super::{initiation_type::Invitee, pairwise_info::PairwiseInfo, trait_bounds::BootstrapDidDoc, Connection};
use crate::{
    common::signing::decode_signed_connection_response,
    errors::error::{AriesVcxError, AriesVcxErrorKind},
};

/// Convenience alias
pub type InviteeConnection<S> = Connection<Invitee, S>;

impl InviteeConnection<Initial> {
    /// Creates a new [`InviteeConnection<Initial>`].
    pub fn new_invitee(source_id: String, pairwise_info: PairwiseInfo) -> Self {
        Self {
            source_id,
            state: Initial,
            pairwise_info,
            initiation_type: Invitee,
        }
    }

    /// Accepts an [`Invitation`] and transitions to [`InviteeConnection<Invited>`].
    ///
    /// # Errors
    ///
    /// Will error out if a DidDoc could not be resolved from the [`Invitation`].
    pub async fn accept_invitation(
        self,
        indy_ledger: &dyn IndyLedgerRead,
        invitation: AnyInvitation,
    ) -> VcxResult<InviteeConnection<Invited>> {
        trace!("Connection::accept_invitation >>> invitation: {:?}", &invitation);

        let did_doc = into_did_doc(indy_ledger, &invitation).await?;
        let state = Invited::new(did_doc, invitation);

        // Convert to `InvitedState`
        Ok(Connection {
            state,
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: Invitee,
        })
    }
}

impl InviteeConnection<Invited> {
    /// Sends a [`Request`] to the inviter and transitions to [`InviteeConnection<Requested>`].
    ///
    /// # Errors
    ///
    /// Will error out if sending the request fails.
    pub async fn send_request<T>(
        self,
        wallet: &Arc<dyn BaseWallet>,
        service_endpoint: Url,
        routing_keys: Vec<String>,
        transport: &T,
    ) -> VcxResult<InviteeConnection<Requested>>
    where
        T: Transport,
    {
        trace!("Connection::send_request");

        let recipient_keys = vec![self.pairwise_info.pw_vk.clone()];

        let id = Uuid::new_v4().to_string();

        let mut did_doc = AriesDidDoc::default();
        did_doc.id = self.pairwise_info.pw_did.to_string();
        did_doc.set_service_endpoint(service_endpoint);
        did_doc.set_routing_keys(routing_keys);
        did_doc.set_recipient_keys(recipient_keys);

        let con_data = ConnectionData::new(self.pairwise_info.pw_did.to_string(), did_doc);
        let content = RequestContent::new(self.source_id.to_string(), con_data);

        let mut decorators = RequestDecorators::default();
        let mut timing = Timing::default();
        timing.out_time = Some(Utc::now());
        decorators.timing = Some(timing);

        // Depending on the invitation type, we set the connection's thread ID
        // and the request parent and thread ID differently.
        //
        // When using a Public or OOB invitation, the invitation's ID (current thread ID)
        // is used as the parent thread ID, while the request ID is set as thread ID.
        //
        // Multiple invitees can use the same invitation in these cases, hence the common
        // parent thread ID and different thread IDs (request IDs are unique).
        //
        // When the invitation is Pairwise, it is designed to be sent to a single invitee.
        // In this case, we reuse the invitation ID (current thread ID) as the thread ID
        // in both the connection and the request.
        let (thread_id, thread) = match &self.state.invitation {
            AnyInvitation::Oob(_) => {
                let mut thread = Thread::new(id.clone());
                thread.pthid = Some(self.state.thread_id().to_owned());

                (id.clone(), thread)
            }
            AnyInvitation::Con(invite) => match invite.content {
                InvitationContent::Public(_) => {
                    let mut thread = Thread::new(id.clone());
                    thread.pthid = Some(self.state.thread_id().to_owned());

                    (id.clone(), thread)
                }
                InvitationContent::Pairwise(_) | InvitationContent::PairwiseDID(_) => {
                    let thread = Thread::new(self.state.thread_id().to_owned());
                    (self.state.thread_id().to_owned(), thread)
                }
            },
        };

        decorators.thread = Some(thread);

        let request = Request::with_decorators(id, content, decorators);

        self.send_message(wallet, &request.into(), transport).await?;

        Ok(Connection {
            state: Requested::new(self.state.did_doc, thread_id),
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: Invitee,
        })
    }
}

impl InviteeConnection<Requested> {
    /// Processes a [`SignedResponse`] from the inviter and transitions to [`InviteeConnection<Responded>`].
    ///
    /// # Errors
    ///
    /// Will error out if:
    ///     * the thread ID of the response does not match the connection thread ID
    ///     * no recipient verkeys are provided in the response.
    ///     * decoding the signed response fails
    pub async fn handle_response<T>(
        self,
        wallet: &Arc<dyn BaseWallet>,
        response: Response,
        transport: &T,
    ) -> VcxResult<InviteeConnection<Responded>>
    where
        T: Transport,
    {
        let is_match = matches_thread_id!(response, self.state.thread_id());

        if !is_match {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!(
                    "Cannot handle message {:?}: thread id does not match, expected {:?}",
                    response,
                    self.state.thread_id()
                ),
            ));
        };

        let keys = &self.state.did_doc.recipient_keys()?;
        let their_vk = keys.first().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            "Cannot handle response: remote verkey not found",
        ))?;

        let did_doc = match decode_signed_connection_response(wallet.as_ref(), response.content, their_vk).await {
            Ok(con_data) => Ok(con_data.did_doc),
            Err(err) => {
                error!("Request DidDoc validation failed! Sending ProblemReport...");

                self.send_problem_report(wallet, &err, self.thread_id(), &self.state.did_doc, transport)
                    .await;

                Err(err)
            }
        }?;

        let state = Responded::new(did_doc, self.state.did_doc, self.state.thread_id);

        Ok(Connection {
            state,
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: Invitee,
        })
    }
}

impl InviteeConnection<Responded> {
    /// Sends an acknowledgement message to the inviter and transitions to [`InviteeConnection<Completed>`].
    ///
    /// # Errors
    ///
    /// Will error out if sending the message fails.
    pub async fn send_ack<T>(
        self,
        wallet: &Arc<dyn BaseWallet>,
        transport: &T,
    ) -> VcxResult<InviteeConnection<Completed>>
    where
        T: Transport,
    {
        let id = Uuid::new_v4().to_string();
        let content = AckContent::new(AckStatus::Ok);

        let mut decorators = AckDecorators::new(Thread::new(self.state.thread_id.clone()));
        let mut timing = Timing::default();
        timing.out_time = Some(Utc::now());
        decorators.timing = Some(timing);

        let msg = Ack::with_decorators(id, content, decorators).into();

        self.send_message(wallet, &msg, transport).await?;

        let state = Completed::new(
            self.state.did_doc,
            self.state.bootstrap_did_doc,
            self.state.thread_id,
            None,
        );

        Ok(Connection {
            state,
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: Invitee,
        })
    }
}

impl<S> InviteeConnection<S>
where
    S: BootstrapDidDoc,
{
    pub fn bootstrap_did_doc(&self) -> &AriesDidDoc {
        self.state.bootstrap_did_doc()
    }
}
