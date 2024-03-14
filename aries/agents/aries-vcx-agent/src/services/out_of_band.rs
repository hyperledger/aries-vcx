use std::sync::Arc;

use aries_vcx::{
    did_doc_sov::{
        extra_fields::{didcommv1::ExtraFieldsDidCommV1, KeyKind},
        service::{didcommv1::ServiceDidCommV1, ServiceSov},
    },
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
    protocols::did_exchange::state_machine::generate_keypair,
};
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use public_key::KeyType;
use uuid::Uuid;

use super::connection::ServiceEndpoint;
use crate::{
    storage::{object_cache::ObjectCache, Storage},
    AgentResult,
};

pub struct ServiceOutOfBand<T> {
    wallet: Arc<T>,
    service_endpoint: ServiceEndpoint,
    out_of_band: Arc<ObjectCache<GenericOutOfBand>>,
}

impl<T: BaseWallet> ServiceOutOfBand<T> {
    pub fn new(wallet: Arc<T>, service_endpoint: ServiceEndpoint) -> Self {
        Self {
            wallet,
            service_endpoint,
            out_of_band: Arc::new(ObjectCache::new("out-of-band")),
        }
    }

    pub async fn create_invitation(&self) -> AgentResult<AriesMessage> {
        let public_key = generate_keypair(self.wallet.as_ref(), KeyType::Ed25519).await?;
        let service = {
            let service_id = Uuid::new_v4().to_string();
            ServiceSov::DIDCommV1(ServiceDidCommV1::new(
                service_id.parse()?,
                self.service_endpoint.to_owned().into(),
                ExtraFieldsDidCommV1::builder()
                    .set_recipient_keys(vec![KeyKind::DidKey(public_key.try_into()?)])
                    .build(),
            )?)
        };
        let sender = OutOfBandSender::create()
            .append_service(&OobService::SovService(service))
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
