pub mod states;

use std::sync::Arc;

use messages::diddoc::aries::diddoc::AriesDidDoc;

use crate::{errors::error::VcxResult, utils::uuid};

use self::states::{initial::InitialState, invited::InvitedState, requested::RequestedState};

use messages::{
    a2a::A2AMessage,
    concepts::ack::Ack,
    protocols::connection::{request::Request, response::SignedResponse},
};

use super::{
    common::states::{complete::CompleteState, responded::RespondedState},
    initiation_type::Invitee,
    pairwise_info::PairwiseInfo,
    trait_bounds::Transport,
    Connection,
};
use crate::{
    common::signing::decode_signed_connection_response,
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    handlers::util::verify_thread_id,
    plugins::wallet::base_wallet::BaseWallet,
};

/// Convenience alias
pub type InviteeConnection<S> = Connection<Invitee, S>;

impl<S> InviteeConnection<S> {
    pub fn new(source_id: String, pairwise_info: PairwiseInfo) -> InviteeConnection<InitialState> {
        Connection {
            source_id,
            state: InitialState,
            pairwise_info,
            initiation_type: Invitee,
        }
    }
}

impl InviteeConnection<InitialState> {
    // This should take an Invitation, but that also implies a DDO resolver
    // for public invitations.
    // Proper signature:
    //      pub fn into_invited(self, invitation: Invitation) -> VcxResult<InviteeConnection<InvitedState>> {

    // We'll accept a DidDoc for now.
    pub fn into_invited(self, did_doc: AriesDidDoc) -> VcxResult<InviteeConnection<InvitedState>> {
        trace!("Connection::into_invited >>> did_doc: {:?}", &did_doc);
        let thread_id = uuid::uuid();
        let state = InvitedState { did_doc, thread_id };

        // Convert to `InvitedState`
        Ok(Connection {
            state,
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: Invitee,
        })
    }
}

impl InviteeConnection<InvitedState> {
    pub async fn send_request<T>(
        self,
        wallet: &Arc<dyn BaseWallet>,
        service_endpoint: String,
        routing_keys: Vec<String>,
        transport: &T,
    ) -> VcxResult<InviteeConnection<RequestedState>>
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

        // Should be properly retrieved from Invitation.
        // Also there's if this Request will just be serialized, it might as well take references.
        let request = request.set_parent_thread_id(&self.state.thread_id);

        // The Invitation gets lost along the way when converting from Invited to Requested
        // in previous implementations. Apart from these thread ID's, it's not used at all.
        //
        // Might as well implement it properly when accepting an Invitation in the `into_invited` method.
        //
        // let request_id = request.id.0.clone();
        //
        // let (request, thread_id) = match &self.state.invitation {
        //     Invitation::Public(_) => (
        //         request
        //             .set_parent_thread_id(&self.thread_id)
        //             .set_thread_id_matching_id(),
        //         request_id,
        //     ),
        //     Invitation::Pairwise(_) => (request.set_thread_id(&self.thread_id), self.thread_id().to_owned()),
        //     Invitation::OutOfBand(invite) => (
        //         request.set_parent_thread_id(&invite.id.0).set_thread_id_matching_id(),
        //         request_id,
        //     ),
        // };

        Self::send_message(
            wallet,
            &self.state.did_doc,
            &request.to_a2a_message(),
            &self.pairwise_info.pw_vk,
            transport,
        )
        .await?;

        Ok(Connection {
            state: RequestedState::new(self.state.did_doc, self.state.thread_id),
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: Invitee,
        })
    }
}

impl InviteeConnection<RequestedState> {
    pub async fn handle_response<T>(
        self,
        wallet: &Arc<dyn BaseWallet>,
        response: SignedResponse,
    ) -> VcxResult<InviteeConnection<RespondedState>>
    where
        T: Transport,
    {
        verify_thread_id(&self.state.thread_id, &A2AMessage::ConnectionResponse(response.clone()))?;

        let keys = &self.state.did_doc.recipient_keys()?;
        let remote_vk = keys.first().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            "Cannot handle response: remote verkey not found",
        ))?;

        let did_doc = decode_signed_connection_response(wallet, response, remote_vk)
            .await
            .map(|response| response.connection.did_doc)?;

        let state = RespondedState::new(did_doc, self.state.thread_id);

        Ok(Connection {
            state,
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: Invitee,
        })
    }
}

impl InviteeConnection<RespondedState> {
    pub async fn send_ack<T>(
        self,
        wallet: &Arc<dyn BaseWallet>,
        transport: &T,
    ) -> VcxResult<InviteeConnection<CompleteState>>
    where
        T: Transport,
    {
        let msg = Ack::create()
            .set_out_time()
            .set_thread_id(&self.state.thread_id)
            .to_a2a_message();

        Self::send_message(wallet, &self.state.did_doc, &msg, &self.pairwise_info.pw_vk, transport).await?;

        let state = CompleteState::new(self.state.did_doc, self.state.thread_id, None);

        Ok(Connection {
            state,
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: Invitee,
        })
    }
}
