pub mod initiation_type;
pub mod invitee;
pub mod inviter;
pub mod pairwise_info;
mod trait_bounds;

use messages::{
    a2a::{protocol_registry::ProtocolRegistry, A2AMessage},
    diddoc::aries::diddoc::AriesDidDoc,
    protocols::discovery::disclose::ProtocolDescriptor,
};
use std::sync::Arc;

use crate::{
    core::profile::profile::Profile,
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    utils::send_message,
};

use self::{pairwise_info::PairwiseInfo, trait_bounds::TheirDidDoc};

use super::{SendClosure, SendClosureConnection};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SideConnectionInfo {
    did: String,
    recipient_keys: Vec<String>,
    routing_keys: Vec<String>,
    service_endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    protocols: Option<Vec<ProtocolDescriptor>>,
}

#[derive(Debug, Serialize)]
struct ConnectionInfo {
    my: SideConnectionInfo,
    their: Option<SideConnectionInfo>,
}

pub struct Connection<I, T, S> {
    source_id: String,
    thread_id: String,
    pairwise_info: PairwiseInfo,
    initiation_type: I,
    transport_type: T,
    state: S,
}

impl<I, T, S> Connection<I, T, S> {
    pub fn pairwise_info(&self) -> &PairwiseInfo {
        &self.pairwise_info
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn thread_id(&self) -> &str {
        &self.thread_id
    }

    pub fn protocols(&self) -> Vec<ProtocolDescriptor> {
        ProtocolRegistry::init().protocols()
    }

    fn send_message_closure_connection(&self, profile: &Arc<dyn Profile>) -> SendClosureConnection {
        trace!("send_message_closure_connection >>>");
        let wallet = profile.inject_wallet();
        Box::new(move |message: A2AMessage, sender_vk: String, did_doc: AriesDidDoc| {
            Box::pin(send_message(wallet, sender_vk, did_doc, message))
        })
    }
}

impl<I, T, S> Connection<I, T, S>
where
    S: TheirDidDoc,
{
    pub fn their_did_doc(&self) -> &AriesDidDoc {
        self.state.their_did_doc()
    }

    pub fn remote_did(&self) -> &str {
        &self.their_did_doc().id
    }

    pub fn remote_vk(&self) -> VcxResult<String> {
        self.their_did_doc()
            .recipient_keys()?
            .first()
            .map(ToOwned::to_owned)
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Can't resolve recipient key from the counterparty diddoc.",
            ))
    }

    pub async fn send_message_closure(
        &self,
        profile: &Arc<dyn Profile>,
        send_message: Option<SendClosureConnection>,
    ) -> VcxResult<SendClosure> {
        trace!("send_message_closure >>>");
        let did_doc = self.their_did_doc().clone();
        let sender_vk = self.pairwise_info().pw_vk.clone();
        let send_message = send_message.unwrap_or(self.send_message_closure_connection(profile));
        Ok(Box::new(move |message: A2AMessage| {
            Box::pin(send_message(message, sender_vk, did_doc))
        }))
    }
}

#[cfg(test)]
/// Only present to illustrate the transition between states for the typestate Connection.
/// Methods are only available to given states (encoded as types) and thus the Connection is guaranteed
/// at compile time not to get into an invalid state.
mod dummy_tests {
    use super::*;

    async fn invitee() {
        let con = Connection::new_invitee(source_id, pairwise_info, did_doc, transport_type)
            .process_invite(invitation)
            .unwrap()
            .send_request(profile, service_endpoint, routing_keys, None)
            .await
            .unwrap()
            .handle_connection_response(wallet, response, send_message)
            .await
            .unwrap()
            .handle_send_ack(send_message)
            .await
            .unwrap();
    }

    async fn inviter() {
        let con = Connection::new_inviter(source_id, pairwise_info, transport_type)
            .create_invite(service_endpoint, routing_keys)
            .process_request(profile, request, service_endpoint, routing_keys, send_message)
            .await
            .unwrap()
            .handle_send_response(send_message)
            .await
            .unwrap()
            .handle_confirmation_message(msg)
            .unwrap();
    }
}
