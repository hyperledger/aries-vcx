pub mod state_machine;
mod states;

use std::sync::Arc;

use crate::handlers::util::verify_thread_id;
use crate::{
    common::signing::sign_connection_response, core::profile::profile::Profile, errors::error::VcxResult,
    plugins::wallet::base_wallet::BaseWallet, protocols::SendClosureConnection,
};

use self::states::complete::CompleteState;
use self::states::initial::InitialState;
use self::states::{invited::InvitedState, requested::RequestedState, responded::RespondedState};
use super::{initiation_type::Inviter, pairwise_info::PairwiseInfo, Connection};
use messages::a2a::A2AMessage;
use messages::protocols::connection::invite::PairwiseInvitation;
use messages::protocols::connection::{
    invite::Invitation,
    request::Request,
    response::{Response, SignedResponse},
};
use messages::protocols::discovery::disclose::{ProtocolDescriptor, Disclose};

pub type InviterConnection<S> = Connection<Inviter, S>;

impl<S> InviterConnection<S> {
    pub fn new_inviter(
        source_id: String,
        pairwise_info: PairwiseInfo,
    ) -> InviterConnection<InitialState> {
        Connection {
            source_id,
            thread_id: String::new(),
            state: InitialState::new(None),
            pairwise_info,
            initiation_type: Inviter,
        }
    }
}

impl InviterConnection<InitialState> {
    pub fn create_invitation(
        self,
        routing_keys: Vec<String>,
        service_endpoint: String,
    ) -> InviterConnection<InvitedState> {
        let invite: PairwiseInvitation = PairwiseInvitation::create()
            .set_id(&self.thread_id)
            .set_label(&self.source_id)
            .set_recipient_keys(vec![self.pairwise_info.pw_vk.clone()])
            .set_routing_keys(routing_keys)
            .set_service_endpoint(service_endpoint);

        let Self {
            source_id,
            thread_id,
            pairwise_info,
            initiation_type,
            state,
        } = self;
        let state = (state, Invitation::Pairwise(invite)).into();

        Connection {
            source_id,
            thread_id,
            pairwise_info,
            initiation_type,
            state,
        }
    }

    pub fn create_invite(
        self,
        service_endpoint: String,
        routing_keys: Vec<String>,
    ) -> InviterConnection<InvitedState> {
        self.create_invitation(routing_keys, service_endpoint)
    }
}

impl InviterConnection<InvitedState> {
    pub fn get_invitation(&self) -> &Invitation {
        &self.state.invitation
    }

    async fn handle_connection_request(
        self,
        wallet: Arc<dyn BaseWallet>,
        request: Request,
        new_pairwise_info: &PairwiseInfo,
        new_routing_keys: Vec<String>,
        new_service_endpoint: String,
        _send_message: SendClosureConnection,
    ) -> VcxResult<InviterConnection<RequestedState>> {
        verify_thread_id(self.thread_id(), &A2AMessage::ConnectionRequest(request.clone()))?;
        request.connection.did_doc.validate()?;

        let signed_response = self
            .build_response(
                &wallet,
                &request,
                new_pairwise_info,
                new_routing_keys,
                new_service_endpoint,
            )
            .await?;

        let state = RequestedState {
            signed_response,
            did_doc: request.connection.did_doc,
            thread_id: request.id.0,
        };

        let Self {
            source_id,
            thread_id,
            pairwise_info,
            initiation_type,
            ..
        } = self;

        Ok(Connection {
            source_id,
            thread_id,
            pairwise_info,
            initiation_type,
            state,
        })
    }

    async fn build_response(
        &self,
        wallet: &Arc<dyn BaseWallet>,
        request: &Request,
        new_pairwise_info: &PairwiseInfo,
        new_routing_keys: Vec<String>,
        new_service_endpoint: String,
    ) -> VcxResult<SignedResponse> {
        let new_recipient_keys = vec![new_pairwise_info.pw_vk.clone()];
        sign_connection_response(
            wallet,
            &self.pairwise_info.clone().pw_vk,
            Response::create()
                .set_did(new_pairwise_info.pw_did.to_string())
                .set_service_endpoint(new_service_endpoint)
                .set_keys(new_recipient_keys, new_routing_keys)
                .ask_for_ack()
                .set_thread_id(&request.get_thread_id())
                .set_out_time(),
        )
        .await
    }

    pub async fn process_request(
        self,
        profile: &Arc<dyn Profile>,
        request: Request,
        service_endpoint: String,
        routing_keys: Vec<String>,
        send_message: Option<SendClosureConnection>,
    ) -> VcxResult<InviterConnection<RequestedState>> {
        trace!(
            "Connection::process_request >>> request: {:?}, service_endpoint: {}, routing_keys: {:?}",
            request,
            service_endpoint,
            routing_keys,
        );

        let send_message = send_message.unwrap_or(self.send_message_closure_connection(profile));
        let new_pairwise_info = PairwiseInfo::create(&profile.inject_wallet()).await?;

        self.handle_connection_request(
            profile.inject_wallet(),
            request,
            &new_pairwise_info,
            routing_keys,
            service_endpoint,
            send_message,
        )
        .await
    }
}

impl InviterConnection<RequestedState> {
    pub async fn handle_send_response(
        self,
        send_message: SendClosureConnection,
    ) -> VcxResult<InviterConnection<RespondedState>> {
        send_message(
            self.state.signed_response.to_a2a_message(),
            self.pairwise_info.pw_vk.clone(),
            self.state.did_doc.clone(),
        )
        .await?;

        let Self {
            source_id,
            thread_id,
            pairwise_info,
            initiation_type,
            state,
        } = self;

        Ok(Connection {
            state: state.into(),
            source_id,
            thread_id,
            pairwise_info,
            initiation_type,
        })
    }

    pub async fn send_response(
        self,
        profile: &Arc<dyn Profile>,
        send_message: Option<SendClosureConnection>,
    ) -> VcxResult<InviterConnection<RespondedState>> {
        trace!("Connection::send_response >>>");
        let send_message = send_message.unwrap_or(self.send_message_closure_connection(profile));
        self.handle_send_response(send_message).await
    }
}

impl InviterConnection<RespondedState> {
    pub fn handle_confirmation_message(self, msg: &A2AMessage) -> VcxResult<InviterConnection<CompleteState>> {
        verify_thread_id(self.thread_id(), msg)?;

        let Self {
            source_id,
            thread_id,
            pairwise_info,
            initiation_type,
            state,
        } = self;

        let state = state.into();

        Ok(Connection {
            source_id,
            thread_id,
            pairwise_info,
            initiation_type,
            state,
        })
    }

    pub fn process_ack(self, message: &A2AMessage) -> VcxResult<InviterConnection<CompleteState>> {
        self.handle_confirmation_message(message)
    }
}


impl InviterConnection<CompleteState> {
    pub fn remote_protocols(&self) -> Option<&[ProtocolDescriptor]> {
        self.state.remote_protocols()
    }

    pub fn handle_disclose(&mut self, disclose: Disclose) {
        self.state.handle_disclose(disclose)
    }
}
