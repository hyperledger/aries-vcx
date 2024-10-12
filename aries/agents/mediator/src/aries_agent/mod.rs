use std::{marker::PhantomData, sync::Arc};

use aries_vcx::{
    handlers::out_of_band::sender::OutOfBandSender,
    messages::msg_fields::protocols::out_of_band::invitation::OobService,
    utils::encryption_envelope::EncryptionEnvelope,
};
use aries_vcx_wallet::{
    errors::error::VcxWalletError,
    wallet::{
        askar::{askar_wallet_config::AskarWalletConfig, key_method::KeyMethod},
        base_wallet::{BaseWallet, ManageWallet},
        structs_io::UnpackMessageOutput,
    },
};
use diddoc_legacy::aries::{diddoc::AriesDidDoc, service::AriesService};
use messages::{
    msg_fields::protocols::{
        connection::{request::Request, response::Response, Connection},
        out_of_band::invitation::Invitation as OOBInvitation,
    },
    AriesMessage,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    persistence::{get_persistence, AccountDetails, MediatorPersistence},
    utils::{prelude::*, structs::VerKey},
};

#[cfg(any(test, feature = "client"))]
pub mod client;
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
impl<T: BaseWallet> AgentBuilder<T> {
    pub async fn new_from_wallet_config(
        config: impl ManageWallet,
    ) -> Result<Agent<impl BaseWallet, sqlx::MySqlPool>, VcxWalletError> {
        let wallet = Arc::new(config.create_wallet().await?);

        info!("Connecting to persistence layer");
        let persistence = Arc::new(get_persistence().await);
        Ok(Agent {
            wallet,
            persistence,
            service: None,
        })
    }
    pub async fn new_demo_agent() -> Result<Agent<impl BaseWallet, sqlx::MySqlPool>, VcxWalletError>
    {
        let config = AskarWalletConfig::new(
            "sqlite://:memory:",
            KeyMethod::Unprotected,
            "",
            &Uuid::new_v4().to_string(),
        );
        Self::new_from_wallet_config(config).await
    }
}

// Utils
impl<T: BaseWallet, P: MediatorPersistence> Agent<T, P> {
    pub fn get_wallet_ref(&self) -> Arc<T> {
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
    ) -> Result<(), VcxWalletError> {
        let did_data = self.wallet.create_and_store_my_did(None, None).await?;
        let service = AriesService {
            id: "#inline".to_owned(),
            type_: "did-communication".to_owned(),
            priority: 0,
            recipient_keys: vec![did_data.verkey().base58()],
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
    ) -> Result<(), VcxWalletError> {
        self.reset_service(routing_keys, service_endpoint).await
    }
    pub fn get_oob_invite(&self) -> Result<OOBInvitation, String> {
        if let Some(service) = &self.service {
            let invitation = OutOfBandSender::create()
                .append_service(&OobService::AriesService(service.clone()))
                .oob;
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
        EncryptionEnvelope::create_from_legacy(
            self.wallet.as_ref(),
            message,
            Some(our_vk),
            their_diddoc,
        )
        .await
        .map_err(string_from_std_error)
    }

    pub async fn auth_and_get_details(
        &self,
        sender_verkey: &Option<VerKey>,
    ) -> Result<AccountDetails, String> {
        let auth_pubkey = sender_verkey
            .as_deref()
            .ok_or("Anonymous sender can't be authenticated")?
            .to_owned();
        let account_details = self
            .persistence
            .get_account_details(&auth_pubkey)
            .await
            .map_err(string_from_std_error)?;
        Ok(account_details)
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
        let did_data = self
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
            did_data.did().into(),
            did_data.verkey().base58(),
            self.service.as_ref().unwrap().service_endpoint.clone(),
            self.service.as_ref().unwrap().routing_keys.clone(),
        )
        .await
        .map_err(|e| e.to_string())?;
        let aries_response = AriesMessage::Connection(Connection::Response(response));
        let their_diddoc = request.content.connection.did_doc;
        let packed_response_envelope = EncryptionEnvelope::create_from_legacy(
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
        self.create_account(auth_pubkey, &did_data.verkey().base58(), &their_diddoc)
            .await?;
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
            .await
            .map_err(string_from_std_error)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use aries_vcx::{
        protocols::oob::oob_invitation_to_legacy_did_doc,
        utils::encryption_envelope::EncryptionEnvelope,
    };
    use aries_vcx_wallet::wallet::askar::AskarWallet;
    use log::info;
    use serde_json::Value;
    use test_utils::mockdata::mock_ledger::MockLedger;

    use super::AgentBuilder;

    #[tokio::test]
    pub async fn test_pack_unpack() {
        let message: Value = serde_json::from_str("{}").unwrap();
        let message_bytes = serde_json::to_vec(&message).unwrap();
        let mut agent = AgentBuilder::<AskarWallet>::new_demo_agent().await.unwrap();
        agent
            .init_service(
                vec![],
                "http://127.0.0.1:8005/aries".to_string().parse().unwrap(),
            )
            .await
            .unwrap();
        let mock_ledger = MockLedger {}; // not good. to be dealt later
        let their_diddoc =
            oob_invitation_to_legacy_did_doc(&mock_ledger, &agent.get_oob_invite().unwrap())
                .await
                .unwrap();
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
