pub mod state_machine;
pub mod states;

use messages::{
    diddoc::aries::diddoc::AriesDidDoc,
    protocols::{
        connection::invite::Invitation,
        discovery::disclose::{Disclose, ProtocolDescriptor},
    },
};

use crate::errors::error::VcxResult;

use self::states::{
    complete::CompleteState, initial::InitialState, invited::InvitedState, requested::RequestedState,
    responded::RespondedState,
};

use std::{collections::HashMap, sync::Arc};

use messages::{
    a2a::A2AMessage,
    concepts::ack::Ack,
    protocols::connection::{problem_report::ProblemReport, request::Request, response::SignedResponse},
};

use super::{initiation_type::Invitee, Connection};
use crate::{
    common::signing::decode_signed_connection_response,
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    handlers::util::verify_thread_id,
    plugins::wallet::base_wallet::BaseWallet,
    protocols::{connection::pairwise_info::PairwiseInfo, SendClosureConnection},
};

/// Convenience alias
pub type InviteeConnection<T, S> = Connection<Invitee, T, S>;

impl<T, S> InviteeConnection<T, S> {
    pub fn new_invitee(
        source_id: String,
        pairwise_info: PairwiseInfo,
        did_doc: AriesDidDoc,
        transport_type: T,
    ) -> InviteeConnection<T, InitialState> {
        Connection {
            source_id,
            thread_id: String::new(),
            state: InitialState::new(None, did_doc),
            pairwise_info,
            initiation_type: Invitee,
            transport_type,
        }
    }
}

impl<T> InviteeConnection<T, InitialState> {
    /// Tries to convert [`InviteeNonMediatedConnection<T, InitialState>`] to [`InviteeNonMediatedConnection<T, InvitedState>`]
    /// by handling a received invitation.
    ///
    /// # Errors
    /// Will error out if the there's no thread ID in the [`Invitation`].
    pub fn handle_invitation(self, invitation: Invitation) -> VcxResult<InviteeConnection<T, InvitedState>> {
        let thread_id = invitation.get_id()?;

        let did_doc = self.state.did_doc;
        let state = InvitedState { invitation, did_doc };

        // Convert to `InvitedState`
        Ok(Connection {
            state,
            thread_id,
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: Invitee,
            transport_type: self.transport_type,
        })
    }

    pub fn process_invite(self, invitation: Invitation) -> VcxResult<InviteeConnection<T, InvitedState>> {
        trace!("Connection::process_invite >>> invitation: {:?}", invitation);
        self.handle_invitation(invitation)
    }
}

impl<T> InviteeConnection<T, InvitedState> {
    pub fn get_invitation(&self) -> &Invitation {
        &self.state.invitation
    }

    fn build_connection_request_msg(
        &self,
        routing_keys: Vec<String>,
        service_endpoint: String,
    ) -> VcxResult<(Request, String)> {
        let recipient_keys = vec![self.pairwise_info.pw_vk.clone()];
        let request = Request::create()
            .set_label(self.source_id.to_string())
            .set_did(self.pairwise_info.pw_did.to_string())
            .set_service_endpoint(service_endpoint)
            .set_keys(recipient_keys, routing_keys)
            .set_out_time();

        let request_id = request.id.0.clone();

        let (request, thread_id) = match &self.state.invitation {
            Invitation::Public(_) => (
                request
                    .set_parent_thread_id(&self.thread_id)
                    .set_thread_id_matching_id(),
                request_id,
            ),
            Invitation::Pairwise(_) => (request.set_thread_id(&self.thread_id), self.thread_id().to_owned()),
            Invitation::OutOfBand(invite) => (
                request.set_parent_thread_id(&invite.id.0).set_thread_id_matching_id(),
                request_id,
            ),
        };
        Ok((request, thread_id))
    }

    /// Tries to convert [`SmConnectionInvitee2<InvitedState>`] to [`SmConnectionInvitee2<RequestedState>`]
    /// by sending a connection request.
    ///
    /// # Errors
    /// Will error out if building or sending the connection request message fails.
    pub async fn send_connection_request(
        self,
        routing_keys: Vec<String>,
        service_endpoint: String,
        send_message: SendClosureConnection,
    ) -> VcxResult<InviteeConnection<T, RequestedState>> {
        let (request, thread_id) = self.build_connection_request_msg(routing_keys, service_endpoint)?;
        let did_doc = self.state.did_doc;

        send_message(
            request.to_a2a_message(),
            self.pairwise_info.pw_vk.clone(),
            did_doc.clone(),
        )
        .await?;

        let state = RequestedState { request, did_doc };

        Ok(Connection {
            state,
            thread_id,
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: Invitee,
            transport_type: self.transport_type,
        })
    }

    pub fn handle_problem_report(self, problem_report: ProblemReport) -> VcxResult<InviteeConnection<T, InitialState>> {
        let Self {
            source_id,
            thread_id,
            pairwise_info,
            transport_type,
            state,
            ..
        } = self;

        let state = InitialState::new(Some(problem_report), state.did_doc);

        Ok(Connection {
            state,
            source_id,
            thread_id,
            pairwise_info,
            initiation_type: Invitee,
            transport_type,
        })
    }
}

impl<T> InviteeConnection<T, RequestedState> {
    /// Returns the first entry from the map for which the message indicates a progressable state.
    pub fn find_message_to_update_state(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        messages
            .into_iter()
            .find(|(_, message)| Self::can_progress_state(message))
    }

    /// Determines whether the message indicates a progressable state.
    pub fn can_progress_state(message: &A2AMessage) -> bool {
        matches!(
            message,
            A2AMessage::ConnectionResponse(_) | A2AMessage::ConnectionProblemReport(_)
        )
    }

    /// Tries to convert [`SmConnectionInvitee2<RequestedState>`] to [`SmConnectionInvitee2<RespondedState>`]
    /// by handling a connection response.
    ///
    /// # Errors
    /// Will error out if the thread ID verification fails, there are no keys in the DidDoc
    /// or decoding the response fails.
    //
    // TODO: Why only convert the state to `InitialState` if the decoding fails?
    // Why not on any other errors?
    pub async fn handle_connection_response(
        self,
        wallet: &Arc<dyn BaseWallet>,
        response: SignedResponse,
        _send_message: SendClosureConnection,
    ) -> VcxResult<InviteeConnection<T, RespondedState>> {
        verify_thread_id(self.thread_id(), &A2AMessage::ConnectionResponse(response.clone()))?;

        let remote_vk: String =
            self.state
                .did_doc
                .recipient_keys()?
                .first()
                .cloned()
                .ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Cannot handle response: remote verkey not found",
                ))?;

        let Self {
            source_id,
            thread_id,
            pairwise_info,
            state,
            transport_type,
            ..
        } = self;

        let state = decode_signed_connection_response(wallet, response.clone(), &remote_vk)
            .await
            .and_then(|response| state.try_into_responded(response))?;

        Ok(Connection {
            state,
            source_id,
            thread_id,
            pairwise_info,
            initiation_type: Invitee,
            transport_type,
        })
    }

    pub fn handle_problem_report(self, problem_report: ProblemReport) -> VcxResult<InviteeConnection<T, InitialState>> {
        let Self {
            source_id,
            thread_id,
            pairwise_info,
            transport_type,
            state,
            ..
        } = self;

        let state = InitialState::new(Some(problem_report), state.did_doc);

        Ok(Connection {
            state,
            source_id,
            thread_id,
            pairwise_info,
            initiation_type: Invitee,
            transport_type,
        })
    }
}

impl<T> InviteeConnection<T, RespondedState> {
    fn build_connection_ack_msg(&self) -> Ack {
        Ack::create().set_out_time().set_thread_id(&self.thread_id)
    }

    pub async fn handle_send_ack(
        self,
        send_message: SendClosureConnection,
    ) -> VcxResult<InviteeConnection<T, CompleteState>> {
        let sender_vk = self.pairwise_info().pw_vk.clone();
        let did_doc = self.state.response.connection.did_doc.clone();

        send_message(self.build_connection_ack_msg().to_a2a_message(), sender_vk, did_doc).await?;

        let Self {
            source_id,
            thread_id,
            pairwise_info,
            state,
            transport_type,
            ..
        } = self;

        let state = CompleteState {
            did_doc: state.did_doc,
            bootstrap_did_doc: state.response.connection.did_doc,
            protocols: None,
        };

        Ok(Connection {
            state,
            source_id,
            thread_id,
            pairwise_info,
            initiation_type: Invitee,
            transport_type,
        })
    }
}

impl<T> InviteeConnection<T, CompleteState> {
    pub fn bootstrap_did_doc(&self) -> &AriesDidDoc {
        &self.state.bootstrap_did_doc
    }

    pub fn remote_protocols(&self) -> Option<&[ProtocolDescriptor]> {
        self.state.remote_protocols()
    }

    pub fn handle_disclose(&mut self, disclose: Disclose) {
        self.state.handle_disclose(disclose)
    }
}
