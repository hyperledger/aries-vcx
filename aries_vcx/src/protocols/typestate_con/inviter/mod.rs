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
    pub fn new_inviter(
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
    /// Creates an [`InviterConnection<InvitedState>`], essentially bypassing the [`InitialState`]
    /// where an [`Invitation`] is created.
    /// 
    /// This is useful for cases where an [`Invitation`] is received by the invitee without
    /// any interaction from the inviter, thus the next logical step is to wait for the invitee
    /// to send a connection request.
    pub fn new_awaiting_request(source_id: String, pairwise_info: PairwiseInfo) -> Self {
        Self {
            source_id,
            state: InvitedState::new(None), // what should the thread ID be in this case???
            pairwise_info,
            initiation_type: Inviter,
        }
    }

    /// As the inviter connection can be started directly in the invited state,
    /// like with a public invitation, we might or might not have a thread ID.
    pub fn opt_thread_id(&self) -> Option<&str> {
        self.state.opt_thread_id()
    }

    // This should ideally belong in the Connection<Inviter,RequestedState>
    // but was placed here to retro-fit the previous API.
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

    // Due to backwards compatibility, we generate the signed response and store that in the state.
    // However, it would be more efficient to store the request and postpone the response generation and
    // signing until the next state, thus taking advantage of the request attributes and avoiding cloning the DidDoc.
    pub async fn handle_request<T>(
        self,
        wallet: &Arc<dyn BaseWallet>,
        request: Request,
        new_service_endpoint: String,
        new_routing_keys: Vec<String>,
        transport: &T,
    ) -> VcxResult<InviterConnection<RequestedState>>
    where
        T: Transport,
    {
        trace!(
            "Connection::process_request >>> request: {:?}, service_endpoint: {}, routing_keys: {:?}",
            request,
            new_service_endpoint,
            new_routing_keys,
        );

        // There must be some other way to validate the thread ID other than cloning the entire Request
        self.state
            .thread_id
            .as_ref()
            .map(|thread_id| verify_thread_id(thread_id, &A2AMessage::ConnectionRequest(request.clone())))
            .unwrap_or(Ok(()))?;

        // If the request's DidDoc validation fails, we generate and send a ProblemReport.
        // We then return early with the provided error.
        if let Err(err) = request.connection.did_doc.validate() {
            error!("Request DidDoc validation failed! Sending ProblemReport...");

            self.send_problem_report(
                wallet,
                &err,
                &request.get_thread_id(),
                &request.connection.did_doc,
                transport,
            )
            .await;

            Err(err)?;
        }

        let new_pairwise_info = PairwiseInfo::create(wallet).await?;
        let did_doc = request.connection.did_doc.clone();

        let signed_response = self
            .build_response(
                wallet,
                &request,
                &new_pairwise_info,
                new_service_endpoint,
                new_routing_keys,
            )
            .await?;

        let state = RequestedState::new(signed_response, did_doc);

        Ok(Connection {
            source_id: self.source_id,
            pairwise_info: new_pairwise_info,
            initiation_type: self.initiation_type,
            state,
        })
    }
}

impl InviterConnection<RequestedState> {
    pub async fn send_response<T>(
        self,
        wallet: &Arc<dyn BaseWallet>,
        transport: &T,
    ) -> VcxResult<InviterConnection<RespondedState>>
    where
        T: Transport,
    {
        trace!(
            "Connection::send_response >>> signed_response: {:?}",
            &self.state.signed_response
        );

        let thread_id = self.state.signed_response.get_thread_id();

        self.send_message(wallet, &self.state.signed_response.to_a2a_message(), transport)
            .await?;

        let state = RespondedState::new(self.state.did_doc, thread_id);

        Ok(Connection {
            state,
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
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
