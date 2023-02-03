pub mod states;

use std::sync::Arc;

use messages::protocols::connection::invite::Invitation;

use crate::{
    common::ledger::transactions::into_did_doc, core::profile::profile::Profile, errors::error::VcxResult,
    protocols::connection::trait_bounds::ThreadId, transport::Transport,
};

use self::states::{invited::Invited, requested::Requested};

use messages::{
    a2a::A2AMessage,
    concepts::ack::Ack,
    protocols::connection::{request::Request, response::SignedResponse},
};

use super::{
    common::states::{complete::Complete, initial::Initial, responded::Responded},
    initiation_type::Invitee,
    pairwise_info::PairwiseInfo,
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

impl InviteeConnection<Initial> {
    pub fn new_invitee(source_id: String, pairwise_info: PairwiseInfo) -> Self {
        Self {
            source_id,
            state: Initial,
            pairwise_info,
            initiation_type: Invitee,
        }
    }

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
            .set_did(self.pairwise_info.pw_did.to_string());

        let (thread_id, request) = match &self.state.invitation {
            Invitation::Public(_) | Invitation::OutOfBand(_) => (
                request.id.0.clone(),
                request
                    .set_parent_thread_id(self.state.thread_id())
                    .set_thread_id_matching_id(),
            ),
            _ => (
                self.state.thread_id().to_owned(),
                request.set_thread_id(self.state.thread_id()),
            ),
        };

        let request = request
            .set_service_endpoint(service_endpoint)
            .set_keys(recipient_keys, routing_keys)
            .set_out_time();

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

        let state = Responded::new(did_doc, self.state.thread_id);

        Ok(Connection {
            state,
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: Invitee,
        })
    }
}

impl InviteeConnection<Responded> {
    pub async fn send_ack<T>(
        self,
        wallet: &Arc<dyn BaseWallet>,
        transport: &T,
    ) -> VcxResult<InviteeConnection<Complete>>
    where
        T: Transport,
    {
        let msg = Ack::create()
            .set_out_time()
            .set_thread_id(&self.state.thread_id)
            .to_a2a_message();

        self.send_message(wallet, &msg, transport).await?;

        let state = Complete::new(self.state.did_doc, self.state.thread_id, None);

        Ok(Connection {
            state,
            source_id: self.source_id,
            pairwise_info: self.pairwise_info,
            initiation_type: Invitee,
        })
    }
}
