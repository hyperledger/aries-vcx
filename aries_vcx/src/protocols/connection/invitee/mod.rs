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

use self::states::{completed::Completed, initial::Initial, invited::Invited, requested::Requested};

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
        indy_ledger: &Arc<dyn IndyLedgerRead>,
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
    pub async fn prepare_request(
        self,
        service_endpoint: Url,
        routing_keys: Vec<String>,
    ) -> VcxResult<InviteeConnection<Requested>> {
        trace!("Connection::prepare_request");

        let recipient_keys = vec![self.pairwise_info.pw_vk.clone()];

        let id = Uuid::new_v4().to_string();

        let mut did_doc = AriesDidDoc::default();
        did_doc.id = self.pairwise_info.pw_did.to_string();
        did_doc.set_service_endpoint(service_endpoint);
        did_doc.set_routing_keys(routing_keys);
        did_doc.set_recipient_keys(recipient_keys);

        let con_data = ConnectionData::new(self.pairwise_info.pw_did.to_string(), did_doc);
        let content = RequestContent::builder()
            .label(self.source_id.to_string())
            .connection(con_data)
            .build();

        let decorators = RequestDecorators::builder().timing(Timing::builder().out_time(Utc::now()).build());

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
        let thread = match &self.state.invitation {
            AnyInvitation::Oob(invite) => Thread::builder().thid(id.clone()).pthid(invite.id.clone()).build(),
            AnyInvitation::Con(invite) => match invite.content {
                InvitationContent::Public(_) => Thread::builder()
                    .thid(id.clone())
                    .pthid(self.state.thread_id().to_owned())
                    .build(),
                InvitationContent::Pairwise(_) | InvitationContent::PairwiseDID(_) => {
                    Thread::builder().thid(self.state.thread_id().to_owned()).build()
                }
            },
        };

        let thread_id = thread.thid.clone();
        let decorators = decorators.thread(thread).build();

        let request = Request::builder()
            .id(id)
            .content(content)
            .decorators(decorators)
            .build();

        Ok(Connection {
            state: Requested::new(self.state.did_doc, thread_id, request),
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
    ) -> VcxResult<InviteeConnection<Completed>>
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

        let did_doc = match decode_signed_connection_response(wallet, response.content, their_vk).await {
            Ok(con_data) => Ok(con_data.did_doc),
            Err(err) => {
                error!("Request DidDoc validation failed! Sending ProblemReport...");

                self.send_problem_report(wallet, &err, self.thread_id(), &self.state.did_doc, transport)
                    .await;

                Err(err)
            }
        }?;

        let state = Completed::new(did_doc, self.state.did_doc, self.state.thread_id, None);

        Ok(Connection {
            state,
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: Invitee,
        })
    }

    pub fn get_request(&self) -> &Request {
        &self.state.request
    }
}

impl InviteeConnection<Completed> {
    /// Sends an acknowledgement message to the inviter and transitions to [`InviteeConnection<Completed>`].
    ///
    /// # Errors
    ///
    /// Will error out if sending the message fails.
    pub fn get_ack(&self) -> Ack {
        let id = Uuid::new_v4().to_string();
        let content = AckContent::builder().status(AckStatus::Ok).build();

        let decorators = AckDecorators::builder()
            .thread(Thread::builder().thid(self.state.thread_id.clone()).build())
            .timing(Timing::builder().out_time(Utc::now()).build())
            .build();

        Ack::builder().id(id).content(content).decorators(decorators).build()
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
