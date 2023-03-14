pub mod states;

use std::sync::Arc;

use messages::{
    a2a::A2AMessage,
    concepts::ack::Ack,
    diddoc::aries::diddoc::AriesDidDoc,
    protocols::connection::{invite::Invitation, request::Request, response::SignedResponse},
};

use self::states::{
    completed::Completed, initial::Initial, invited::Invited, requested::Requested, responded::Responded,
};
use super::{initiation_type::Invitee, pairwise_info::PairwiseInfo, trait_bounds::BootstrapDidDoc, Connection};
use crate::{
    common::{ledger::transactions::into_did_doc, signing::decode_signed_connection_response},
    core::profile::profile::Profile,
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    handlers::util::verify_thread_id,
    plugins::wallet::base_wallet::BaseWallet,
    protocols::connection::trait_bounds::ThreadId,
    transport::Transport,
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
        profile: &Arc<dyn Profile>,
        invitation: Invitation,
    ) -> VcxResult<InviteeConnection<Invited>> {
        trace!("Connection::accept_invitation >>> invitation: {:?}", &invitation);

        let did_doc = into_did_doc(profile, &invitation).await?;
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
        service_endpoint: String,
        routing_keys: Vec<String>,
        transport: &T,
    ) -> VcxResult<InviteeConnection<Requested>>
    where
        T: Transport,
    {
        trace!("Connection::send_request");

        let recipient_keys = vec![self.pairwise_info.pw_vk.clone()];

        let request = Request::create()
            .set_label(self.source_id.to_string())
            .set_did(self.pairwise_info.pw_did.to_string())
            .set_service_endpoint(service_endpoint)
            .set_keys(recipient_keys, routing_keys)
            .set_out_time();

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
        let (thread_id, request) = match &self.state.invitation {
            Invitation::Public(_) | Invitation::OutOfBand(_) => (
                request.id.0.clone(),
                request
                    .set_parent_thread_id(self.state.thread_id())
                    .set_thread_id_matching_id(),
            ),
            Invitation::Pairwise(_) => (
                self.state.thread_id().to_owned(),
                request.set_thread_id(self.state.thread_id()),
            ),
        };

        self.send_message(wallet, &request.to_a2a_message(), transport).await?;

        Ok(Connection {
            state: Requested::new(self.state.did_doc, thread_id),
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: Invitee,
        })
    }
}

impl InviteeConnection<Requested> {
    /// Processes a [`SignedResponse`] from the inviter and transitions to
    /// [`InviteeConnection<Responded>`].
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
        response: SignedResponse,
        transport: &T,
    ) -> VcxResult<InviteeConnection<Responded>>
    where
        T: Transport,
    {
        verify_thread_id(
            self.state.thread_id(),
            &A2AMessage::ConnectionResponse(response.clone()),
        )?;

        let keys = &self.state.did_doc.recipient_keys()?;
        let their_vk = keys.first().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            "Cannot handle response: remote verkey not found",
        ))?;

        let did_doc = match decode_signed_connection_response(wallet, response, their_vk).await {
            Ok(response) => Ok(response.connection.did_doc),
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
    /// Sends an acknowledgement message to the inviter and transitions to
    /// [`InviteeConnection<Completed>`].
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
        let msg = Ack::create()
            .set_out_time()
            .set_thread_id(&self.state.thread_id)
            .to_a2a_message();

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
