use aries_vcx::handlers::out_of_band::sender::OutOfBandSender;
use aries_vcx::messages::msg_fields::protocols::out_of_band::invitation::OobService;
use aries_vcx_core::errors::error::AriesVcxCoreError;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use aries_vcx_core::wallet::indy::wallet::create_and_open_wallet;
use aries_vcx_core::wallet::indy::{IndySdkWallet, WalletConfig};
use aries_vcx_core::WalletHandle;
use diddoc_legacy::aries::service::AriesService;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;

// #[cfg(test)]
pub mod client;

#[derive(Debug, Clone)]
pub struct Agent<T>
where
    T: BaseWallet,
{
    wallet: T,
    service: Option<AriesService>,
}

/// Constructors
impl Agent<IndySdkWallet> {
    pub async fn new_from_wallet_config(config: WalletConfig) -> Result<Self, AriesVcxCoreError> {
        let wallet_handle: WalletHandle = create_and_open_wallet(&config).await?;
        let wallet = IndySdkWallet::new(wallet_handle);
        Ok(Self { wallet, service: None })
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
}
