use std::{marker::PhantomData, sync::Arc};

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
use diddoc_legacy::aries::{diddoc::AriesDidDoc, service::AriesService};
use mediation::storage::{get_persistence, MediatorPersistence};
use messages::{
    msg_fields::protocols::{
        connection::{request::Request, response::Response, Connection},
        out_of_band::invitation::Invitation as OOBInvitation,
    },
    AriesMessage,
};
use serde_json::json;

use self::transports::AriesTransport;
use crate::utils::{prelude::*, structs::VerKey};

#[cfg(any(test, feature = "client"))]
pub mod client;
pub mod transports;
pub mod utils;

#[derive(Clone)]
pub struct Agent<T: BaseWallet, P: MediatorPersistence> {
    wallet: Arc<T>,
    persistence: Arc<P>,
    service: Option<AriesService>,
}

pub type ArcAgent<T, P> = Arc<Agent<T, P>>;

pub struct AgentBuilder<T: BaseWallet> {
    _type_wallet: PhantomData<T>,
}
/// Constructors
impl AgentBuilder<IndySdkWallet> {
    pub async fn new_from_wallet_config(
        config: WalletConfig,
    ) -> Result<Agent<IndySdkWallet, sqlx::MySqlPool>, AriesVcxCoreError> {
        let wallet_handle: WalletHandle = create_and_open_wallet(&config).await?;
        let wallet = Arc::new(IndySdkWallet::new(wallet_handle));
        info!("Connecting to persistence layer");
        let persistence = Arc::new(get_persistence().await);
        Ok(Agent {
            wallet,
            persistence,
            service: None,
        })
    }
    pub async fn new_demo_agent() -> Result<Agent<IndySdkWallet, sqlx::MySqlPool>, AriesVcxCoreError>
    {
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
impl<T: BaseWallet + 'static, P: MediatorPersistence> Agent<T, P> {
    pub fn get_wallet_ref(&self) -> Arc<impl BaseWallet> {
        self.wallet.clone()
    }
    pub fn get_persistence_ref(&self) -> Arc<impl MediatorPersistence> {
        self.persistence.clone()
    }
    pub fn get_service_ref(&self) -> Option<&AriesService> {
        self.service.as_ref()
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

    pub async fn pack_didcomm(
        &self,
        message: &[u8],
        our_vk: &VerKey,
        their_diddoc: &AriesDidDoc,
    ) -> Result<EncryptionEnvelope, String> {
        EncryptionEnvelope::create(self.wallet.as_ref(), message, Some(our_vk), their_diddoc)
            .await
            .map_err(string_from_std_error)
    }
    pub async fn pack_and_send_didcomm(
        &self,
        message: &[u8],
        our_vk: &VerKey,
        their_diddoc: &AriesDidDoc,
        aries_transport: &mut impl AriesTransport,
    ) -> Result<(), String> {
        let EncryptionEnvelope(packed_message) =
            self.pack_didcomm(message, our_vk, their_diddoc).await?;
        let packed_json = serde_json::from_slice(&packed_message).map_err(string_from_std_error)?;
        info!(
            "Packed: {:?}, sending",
            serde_json::to_string(&packed_json).unwrap()
        );
        aries_transport
            .push_aries_envelope(packed_json, their_diddoc)
            .await
            .map_err(string_from_std_error)
    }

    pub async fn auth_and_get_details(
        &self,
        sender_verkey: &Option<VerKey>,
    ) -> Result<(String, VerKey, VerKey, AriesDidDoc), String> {
        let auth_pubkey = sender_verkey
            .as_deref()
            .ok_or("Anonymous sender can't be authenticated")?
            .to_owned();
        let (_sr_no, account_name, our_signing_key, did_doc_json) =
            self.persistence.get_account_details(&auth_pubkey).await?;
        let diddoc =
            serde_json::from_value::<AriesDidDoc>(did_doc_json).map_err(string_from_std_error)?;
        Ok((account_name, auth_pubkey, our_signing_key, diddoc))
    }
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
            self.wallet.as_ref(),
            thread_id,
            old_vk.clone(),
            did,
            vk.clone(),
            self.service.as_ref().unwrap().service_endpoint.clone(),
            self.service.as_ref().unwrap().routing_keys.clone(),
        )
        .await
        .map_err(|e| e.to_string())?;
        let aries_response = AriesMessage::Connection(Connection::Response(response));
        let their_diddoc = request.content.connection.did_doc;
        let packed_response_envelope = EncryptionEnvelope::create(
            self.wallet.as_ref(),
            json!(aries_response).to_string().as_bytes(),
            Some(&old_vk),
            &their_diddoc,
        )
        .await
        .map_err(|e| e.to_string())?;
        let their_keys = their_diddoc.recipient_keys().map_err(|e| e.to_string())?;
        let auth_pubkey = their_keys
            .first()
            .ok_or("No recipient key for client :/ ?".to_owned())?;
        self.create_account(auth_pubkey, &vk, &their_diddoc).await?;
        Ok(packed_response_envelope)
    }

    pub async fn create_account(
        &self,
        their_vk: &VerKey,
        our_vk: &VerKey,
        did_doc: &AriesDidDoc,
    ) -> Result<(), String> {
        self.persistence
            .create_account(their_vk, our_vk, &json!(did_doc).to_string())
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use aries_vcx::utils::encryption_envelope::EncryptionEnvelope;
    use log::info;
    use serde_json::Value;

    use super::AgentBuilder;
    use crate::aries_agent::utils::oob2did;

    #[tokio::test]
    pub async fn test_pack_unpack() {
        let message: Value = serde_json::from_str("{}").unwrap();
        let message_bytes = serde_json::to_vec(&message).unwrap();
        let mut agent = AgentBuilder::new_demo_agent().await.unwrap();
        agent
            .init_service(
                vec![],
                "http://127.0.0.1:8005/aries".to_string().parse().unwrap(),
            )
            .await
            .unwrap();
        let their_diddoc = oob2did(agent.get_oob_invite().unwrap());
        let our_service = agent.service.as_ref().unwrap();
        let our_vk = our_service.recipient_keys.first().unwrap();
        let EncryptionEnvelope(packed) = agent
            .pack_didcomm(&message_bytes, our_vk, &their_diddoc)
            .await
            .unwrap();
        let unpacked = agent.unpack_didcomm(&packed).await.unwrap();
        info!("{:?}", unpacked);
        print!("{:?}", unpacked);
    }
}
