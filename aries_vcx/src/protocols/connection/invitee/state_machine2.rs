use std::clone::Clone;
use std::collections::HashMap;
use std::sync::Arc;

use crate::common::signing::decode_signed_connection_response;
use crate::errors::error::prelude::*;
use crate::handlers::util::verify_thread_id;
use crate::plugins::wallet::base_wallet::BaseWallet;
use crate::protocols::connection::invitee::states::complete::CompleteState;
use crate::protocols::connection::invitee::states::initial::InitialState;
use crate::protocols::connection::invitee::states::invited::InvitedState;
use crate::protocols::connection::invitee::states::requested::RequestedState;
use crate::protocols::connection::invitee::states::responded::RespondedState;
use crate::protocols::connection::pairwise_info::PairwiseInfo;
use crate::protocols::SendClosureConnection;
use messages::a2a::protocol_registry::ProtocolRegistry;
use messages::a2a::A2AMessage;
use messages::concepts::ack::Ack;
use messages::diddoc::aries::diddoc::AriesDidDoc;
use messages::protocols::connection::invite::Invitation;
use messages::protocols::connection::problem_report::ProblemReport;
use messages::protocols::connection::request::Request;
use messages::protocols::connection::response::SignedResponse;
use messages::protocols::discovery::disclose::{Disclose, ProtocolDescriptor};

use super::states::InviteeState as InviteeStateTrait;

#[derive(Clone, Serialize, Deserialize)]
pub struct SmConnectionInvitee2<T: InviteeStateTrait> {
    source_id: String,
    thread_id: String,
    pairwise_info: PairwiseInfo,
    state: T,
}

impl<T> SmConnectionInvitee2<T>
where
    T: InviteeStateTrait,
{
    pub fn new(
        source_id: &str,
        pairwise_info: PairwiseInfo,
        did_doc: AriesDidDoc,
    ) -> SmConnectionInvitee2<InitialState> {
        SmConnectionInvitee2 {
            source_id: source_id.to_string(),
            thread_id: String::new(),
            state: InitialState::new(None, Some(did_doc)),
            pairwise_info,
        }
    }

    pub fn from_parts(source_id: String, thread_id: String, pairwise_info: PairwiseInfo, state: T) -> Self {
        Self {
            source_id,
            thread_id,
            pairwise_info,
            state,
        }
    }

    pub fn pairwise_info(&self) -> &PairwiseInfo {
        &self.pairwise_info
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn get_thread_id(&self) -> String {
        self.thread_id.clone()
    }

    // Method could be propagated to the trait, and return
    // a certain enum variant based on the state.
    //
    // pub fn get_state(&self) -> InviteeState {
    //     InviteeState::from(self.state.clone())
    // }

    pub fn state_object(&self) -> &T {
        &self.state
    }

    pub fn their_did_doc(&self) -> Option<AriesDidDoc> {
        self.state.their_did_doc()
    }

    pub fn bootstrap_did_doc(&self) -> Option<AriesDidDoc> {
        self.state.bootstrap_did_doc()
    }

    pub fn get_protocols(&self) -> Vec<ProtocolDescriptor> {
        ProtocolRegistry::init().protocols()
    }

    pub fn remote_did(&self) -> VcxResult<String> {
        self.their_did_doc()
            .map(|did_doc: AriesDidDoc| did_doc.id)
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Remote Connection DID is not set",
            ))
    }

    pub fn remote_vk(&self) -> VcxResult<String> {
        let did_did = self.their_did_doc().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::NotReady,
            "Counterparty diddoc is not available.",
        ))?;

        did_did
            .recipient_keys()?
            .get(0)
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Can't resolve recipient key from the counterparty diddoc.",
            ))
            .map(|s| s.to_string())
    }
}

impl SmConnectionInvitee2<InitialState> {
    /// Tries to convert [`SmConnectionInvitee2<InitialState>`] to [`SmConnectionInvitee<InvitedState>`]
    /// by handling a received invitation.
    ///
    /// # Errors
    /// Will error out if the there's no thread ID in the [`Invitation`]
    /// or if there's no [`AriesDidDoc`] in the [`InitialState`].
    pub fn handle_invitation(self, invitation: Invitation) -> VcxResult<SmConnectionInvitee2<InvitedState>> {
        let thread_id = invitation.get_id()?;

        // Lazy error creation through a closure is better.
        let did_doc = self.state.did_doc.ok_or_else(|| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Expected none None state.did_doc result given current state",
            )
        })?;

        let state = InvitedState { invitation, did_doc };

        // Convert to `InvitedState`
        Ok(SmConnectionInvitee2 {
            state,
            thread_id,
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
        })
    }
}

impl SmConnectionInvitee2<InvitedState> {
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
            Invitation::Pairwise(_) => (request.set_thread_id(&self.thread_id), self.get_thread_id()),
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
    ) -> VcxResult<SmConnectionInvitee2<RequestedState>> {
        let (request, thread_id) = self.build_connection_request_msg(routing_keys, service_endpoint)?;
        let did_doc = self.state.did_doc;

        send_message(
            request.to_a2a_message(),
            self.pairwise_info.pw_vk.clone(),
            did_doc.clone(),
        )
        .await?;

        let state = RequestedState { request, did_doc };

        Ok(SmConnectionInvitee2 {
            state,
            thread_id,
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
        })
    }

    // TODO: Maybe pass in the problem_report???
    pub fn handle_problem_report(
        self,
        _problem_report: ProblemReport,
    ) -> VcxResult<SmConnectionInvitee2<InitialState>> {
        let state = InitialState::new(None, None);
        let Self {
            source_id,
            thread_id,
            pairwise_info,
            ..
        } = self;

        Ok(SmConnectionInvitee2 {
            state,
            source_id,
            thread_id,
            pairwise_info,
        })
    }
}

impl SmConnectionInvitee2<RequestedState> {
    /// Returns the first entry from the map for which the message indicates a progressable state.
    pub fn find_message_to_update_state(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        messages
            .into_iter()
            .filter(|(_, message)| Self::can_progress_state(message))
            .next()
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
    ) -> VcxResult<SmConnectionInvitee2<RespondedState>> {
        verify_thread_id(&self.get_thread_id(), &A2AMessage::ConnectionResponse(response.clone()))?;

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
        } = self;

        let state = decode_signed_connection_response(wallet, response.clone(), &remote_vk)
            .await
            .and_then(|response| state.try_into_responded(response))?;

        Ok(SmConnectionInvitee2 {
            state,
            source_id,
            thread_id,
            pairwise_info,
        })
    }

    pub fn handle_problem_report(
        self,
        _problem_report: ProblemReport,
    ) -> VcxResult<SmConnectionInvitee2<InitialState>> {
        let state = InitialState::new(None, None);
        let Self {
            source_id,
            thread_id,
            pairwise_info,
            ..
        } = self;

        Ok(SmConnectionInvitee2 {
            state,
            source_id,
            thread_id,
            pairwise_info,
        })
    }
}

impl SmConnectionInvitee2<RespondedState> {
    fn build_connection_ack_msg(&self) -> Ack {
        Ack::create().set_out_time().set_thread_id(&self.thread_id)
    }

    pub async fn handle_send_ack(
        self,
        send_message: SendClosureConnection,
    ) -> VcxResult<SmConnectionInvitee2<CompleteState>> {
        let sender_vk = self.pairwise_info().pw_vk.clone();
        let did_doc = self.state.response.connection.did_doc.clone();

        send_message(self.build_connection_ack_msg().to_a2a_message(), sender_vk, did_doc).await?;

        let Self {
            source_id,
            thread_id,
            pairwise_info,
            state,
        } = self;

        let state = CompleteState {
            did_doc: state.did_doc,
            bootstrap_did_doc: state.response.connection.did_doc,
            protocols: None,
        };

        Ok(SmConnectionInvitee2 {
            state,
            source_id,
            thread_id,
            pairwise_info,
        })
    }
}

impl SmConnectionInvitee2<CompleteState> {
    pub fn get_remote_protocols(&self) -> Option<Vec<ProtocolDescriptor>> {
        self.state.protocols.clone()
    }

    pub fn handle_disclose(&mut self, disclose: Disclose) {
        self.state.protocols = Some(disclose.protocols);
    }
}
