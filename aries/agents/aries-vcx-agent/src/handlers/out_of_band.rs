use std::sync::Arc;

use aries_vcx::{
    handlers::out_of_band::{
        receiver::OutOfBandReceiver, sender::OutOfBandSender, GenericOutOfBand,
    },
    messages::{
        msg_fields::protocols::out_of_band::invitation::{Invitation as OobInvitation, OobService},
        msg_types::{
            protocols::did_exchange::{DidExchangeType, DidExchangeTypeV1},
            Protocol,
        },
        AriesMessage,
    },
    protocols::did_exchange::state_machine::create_our_did_document,
};
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use did_peer::peer_did::{numalgos::numalgo2::Numalgo2, PeerDid};

use super::connection::ServiceEndpoint;
use crate::{
    storage::{agent_storage_inmem::AgentStorageInMem, AgentStorage},
    AgentResult,
};

pub struct ServiceOutOfBand<T> {
    wallet: Arc<T>,
    service_endpoint: ServiceEndpoint,
    out_of_band: Arc<AgentStorageInMem<GenericOutOfBand>>,
}

impl<T: BaseWallet> ServiceOutOfBand<T> {
    pub fn new(wallet: Arc<T>, service_endpoint: ServiceEndpoint) -> Self {
        Self {
            wallet,
            service_endpoint,
            out_of_band: Arc::new(AgentStorageInMem::new("out-of-band")),
        }
    }

    pub async fn create_invitation(&self) -> AgentResult<AriesMessage> {
        let (our_did_document, _our_verkey) =
            create_our_did_document(self.wallet.as_ref(), self.service_endpoint.clone(), vec![])
                .await?;
        let peer_did = PeerDid::<Numalgo2>::from_did_doc(our_did_document)?;

        let sender = OutOfBandSender::create()
            .append_service(&OobService::Did(peer_did.to_string()))
            .append_handshake_protocol(Protocol::DidExchangeType(DidExchangeType::V1(
                DidExchangeTypeV1::new_v1_0(),
            )))?;

        self.out_of_band.insert(
            &sender.get_id(),
            GenericOutOfBand::Sender(sender.to_owned()),
        )?;

        Ok(sender.to_aries_message())
    }

    pub fn receive_invitation(&self, invitation: AriesMessage) -> AgentResult<String> {
        let receiver = OutOfBandReceiver::create_from_a2a_msg(&invitation)?;

        self.out_of_band
            .insert(&receiver.get_id(), GenericOutOfBand::Receiver(receiver))
    }

    pub fn get_invitation(&self, invitation_id: &str) -> AgentResult<OobInvitation> {
        let out_of_band = self.out_of_band.get(invitation_id)?;
        match out_of_band {
            GenericOutOfBand::Sender(sender) => Ok(sender.oob),
            GenericOutOfBand::Receiver(receiver) => Ok(receiver.oob),
        }
    }

    pub fn exists_by_id(&self, thread_id: &str) -> bool {
        self.out_of_band.contains_key(thread_id)
    }
}
