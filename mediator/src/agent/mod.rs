use std::sync::Arc;

use aries_vcx::{
    handlers::out_of_band::sender::OutOfBandSender,
    messages::msg_fields::protocols::out_of_band::invitation::OobService,
    utils::encryption_envelope::EncryptionEnvelope,
};
use aries_vcx_core::{
    errors::error::AriesVcxCoreError,
    wallet::{
        base_wallet::BaseWallet,
        indy::{wallet::create_and_open_wallet, IndySdkWallet, WalletConfig},
        structs_io::UnpackMessageOutput,
    },
    WalletHandle,
};
use diddoc_legacy::aries::service::AriesService;
use messages::{
    msg_fields::protocols::{
        connection::{request::Request, response::Response, Connection},
        out_of_band::invitation::Invitation as OOBInvitation,
    },
    AriesMessage,
};
use xum_test_server::storage::{get_persistence, MediatorPersistence};

use crate::utils::prelude::*;

pub mod utils;
// #[cfg(test)]
pub mod client;

#[derive(Clone)]
pub struct Agent<T>
where
    T: BaseWallet,
{
    wallet: T,
    wallet_ref: Arc<dyn BaseWallet>,
    persistence: Arc<dyn MediatorPersistence>,
    service: Option<AriesService>,
}

/// Constructors
impl Agent<IndySdkWallet> {
    pub async fn new_from_wallet_config(config: WalletConfig) -> Result<Self, AriesVcxCoreError> {
        let wallet_handle: WalletHandle = create_and_open_wallet(&config).await?;
        let wallet = IndySdkWallet::new(wallet_handle);
        let wallet_ref = Arc::new(IndySdkWallet::new(wallet_handle));
        let persistence = Arc::new(get_persistence().await);
        Ok(Self {
            wallet,
            wallet_ref,
            persistence,
            service: None,
        })
    }
    pub async fn new_demo_agent() -> Result<Self, AriesVcxCoreError> {
        let config = WalletConfig {
            wallet_name: uuid::Uuid::new_v4().to_string(),
            wallet_key: "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY".into(),
            wallet_key_derivation: "RAW".into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };
        Self::new_from_wallet_config(config).await
    }
}

// Utils
impl<T> Agent<T>
where
    T: BaseWallet,
{
    pub fn get_wallet_ref(&self) -> Arc<dyn BaseWallet> {
        self.wallet_ref.clone()
    }

    pub async fn reset_service(
        &mut self,
        routing_keys: Vec<String>,
        service_endpoint: url::Url,
    ) -> Result<(), AriesVcxCoreError> {
        let (_, vk) = self.wallet.create_and_store_my_did(None, None).await?;
        let service = AriesService {
            id: "#inline".to_owned(),
            type_: "did-communication".to_owned(),
            priority: 0,
            recipient_keys: vec![vk],
            routing_keys,
            service_endpoint,
        };
        self.service = Some(service);
        Ok(())
    }

    pub async fn init_service(
        &mut self,
        routing_keys: Vec<String>,
        service_endpoint: url::Url,
    ) -> Result<(), AriesVcxCoreError> {
        self.reset_service(routing_keys, service_endpoint).await
    }
    pub fn get_oob_invite(&self) -> Result<OOBInvitation, String> {
        if let Some(service) = &self.service {
            let invitation = OutOfBandSender::create()
                .append_service(&OobService::AriesService(service.clone()))
                .oob
                .clone();
            Ok(invitation)
        } else {
            Err("No service to create invite for".to_owned())
        }
    }
    pub async fn unpack_didcomm(&self, didcomm_msg: &[u8]) -> Result<UnpackMessageOutput, String> {
        let unpacked = self
            .wallet
            .unpack_message(didcomm_msg)
            .await
            .expect("Valid didcomm?");
        info!("{:#?}", unpacked);
        Ok(unpacked)
    }
    // pub async fn pack_message(&self, message: AriesMessage, recipient_vk: VeriKey, sender_vk:
    // VeriKey) -> Value {     todo!()
    // }
    pub async fn handle_connection_req(
        &self,
        request: Request,
    ) -> Result<EncryptionEnvelope, String> {
        if let Err(err) = request.content.connection.did_doc.validate() {
            return Err(format!("Request DidDoc validation failed! {:?}", err));
        }

        let thread_id = request
            .decorators
            .thread
            .map(|t| t.thid)
            .unwrap_or(request.id);
        let (did, vk) = self
            .wallet
            .create_and_store_my_did(None, None)
            .await
            .map_err(|e| e.to_string())?;
        let old_vk = self
            .service
            .as_ref()
            .unwrap()
            .recipient_keys
            .first()
            .unwrap()
            .to_owned();

        let response: Response = utils::build_response_content(
            &self.wallet_ref,
            thread_id,
            old_vk.clone(),
            did,
            vk,
            self.service.as_ref().unwrap().service_endpoint.clone(),
            self.service.as_ref().unwrap().routing_keys.clone(),
        )
        .await
        .map_err(|e| e.to_string())?;
        let aries_response = AriesMessage::Connection(Connection::Response(response));
        let their_diddoc = request.content.connection.did_doc;
        let packed_response_envelope = EncryptionEnvelope::create(
            &self.get_wallet_ref(),
            &aries_response,
            Some(&old_vk),
            &their_diddoc,
        )
        .await
        .map_err(|e| e.to_string())?;
        Ok(packed_response_envelope)
    }
}
