pub mod states;

use std::sync::Arc;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind};
use crate::handlers::util::verify_thread_id;
use crate::utils::uuid;
use crate::{
    common::signing::sign_connection_response, errors::error::VcxResult, plugins::wallet::base_wallet::BaseWallet,
};

use self::states::initial::InitialState;
use self::states::{invited::InvitedState, requested::RequestedState};
use super::common::states::complete::CompleteState;
use super::common::states::responded::RespondedState;
use super::traits::Transport;
use super::{initiation_type::Inviter, pairwise_info::PairwiseInfo, Connection};
use messages::a2a::A2AMessage;
use messages::protocols::connection::invite::PairwiseInvitation;
use messages::protocols::connection::{
    invite::Invitation,
    request::Request,
    response::{Response, SignedResponse},
};

pub type InviterConnection<S> = Connection<Inviter, S>;

impl InviterConnection<InitialState> {
    pub fn new(
        source_id: String,
        pairwise_info: PairwiseInfo,
        routing_keys: Vec<String>,
        service_endpoint: String,
    ) -> Self {
        let invite: PairwiseInvitation = PairwiseInvitation::create()
            .set_id(&uuid::uuid())
            .set_label(&source_id)
            .set_recipient_keys(vec![pairwise_info.pw_vk.clone()])
            .set_routing_keys(routing_keys)
            .set_service_endpoint(service_endpoint);

        let invitation = Invitation::Pairwise(invite);

        Self {
            source_id,
            state: InitialState::new(invitation),
            pairwise_info,
            initiation_type: Inviter,
        }
    }

    pub fn get_invitation(&self) -> &Invitation {
        &self.state.invitation
    }

    pub fn send_invitation<T>(self, _transport: &T) -> VcxResult<InviterConnection<InvitedState>> {
        // Implement some way to actually send the invitation
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::ActionNotSupported,
            "sending invites isn't yet supported!",
        ))
    }
}

impl InviterConnection<InvitedState> {
    pub fn new_invited(source_id: String, pairwise_info: PairwiseInfo) -> Self {
        Self {
            source_id,
            state: InvitedState::new(None), // what should the thread ID be in this case???
            pairwise_info,
            initiation_type: Inviter,
        }
    }

    pub async fn handle_request(self, request: Request) -> VcxResult<InviterConnection<RequestedState>> {
        trace!("Connection::process_request >>> request: {:?}", request,);

        // There must be some other way to validate the thread ID other than cloning the entire Request
        self.state
            .thread_id
            .as_ref()
            .map(|thread_id| verify_thread_id(thread_id, &A2AMessage::ConnectionRequest(request.clone())))
            .unwrap_or(Ok(()))?;

        request.connection.did_doc.validate()?;

        let state = RequestedState::new(request);

        Ok(Connection {
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: self.initiation_type,
            state,
        })
    }
}

impl InviterConnection<RequestedState> {
    async fn build_response(
        &self,
        wallet: &Arc<dyn BaseWallet>,
        request: &Request,
        new_pairwise_info: &PairwiseInfo,
        new_service_endpoint: String,
        new_routing_keys: Vec<String>,
    ) -> VcxResult<SignedResponse> {
        let new_recipient_keys = vec![new_pairwise_info.pw_vk.clone()];
        let response = Response::create()
            .set_did(new_pairwise_info.pw_did.to_string())
            .set_service_endpoint(new_service_endpoint)
            .set_keys(new_recipient_keys, new_routing_keys)
            .ask_for_ack()
            .set_thread_id(&request.get_thread_id())
            .set_out_time();

        sign_connection_response(wallet, &self.pairwise_info.pw_vk, response).await
    }

    pub async fn send_response<T>(
        self,
        wallet: &Arc<dyn BaseWallet>,
        new_service_endpoint: String,
        new_routing_keys: Vec<String>,
        transport: &T,
    ) -> VcxResult<InviterConnection<RespondedState>>
    where
        T: Transport,
    {
        trace!(
            "Connection::send_response >>> service_endpoint: {}, routing_keys: {:?}",
            new_service_endpoint,
            new_routing_keys,
        );

        let new_pairwise_info = PairwiseInfo::create(wallet).await?;
        let thread_id = self.state.request.get_thread_id();

        let signed_response = self
            .build_response(
                wallet,
                &self.state.request,
                &new_pairwise_info,
                new_service_endpoint,
                new_routing_keys,
            )
            .await?;

        self.send_message(wallet, &signed_response.to_a2a_message(), transport)
            .await?;

        let state = RespondedState::new(self.state.request.connection.did_doc, thread_id);

        Ok(Connection {
            state,
            source_id: self.source_id,
            pairwise_info: new_pairwise_info,
            initiation_type: self.initiation_type,
        })
    }
}

impl InviterConnection<RespondedState> {
    pub fn acknowledge_connection(self, msg: &A2AMessage) -> VcxResult<InviterConnection<CompleteState>> {
        verify_thread_id(&self.state.thread_id, msg)?;
        let state = CompleteState::new(self.state.did_doc, self.state.thread_id, None);

        Ok(Connection {
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: self.initiation_type,
            state,
        })
    }
}
