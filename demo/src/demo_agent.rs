use aries_vcx_core::wallet::base_wallet::BaseWallet;
use aries_vcx_core::wallet::indy::wallet::create_and_open_wallet;
use aries_vcx_core::wallet::indy::{IndySdkWallet, WalletConfig};
use std::sync::Arc;
use url::Url;

pub struct DemoAgent {
    pub wallet: Arc<dyn BaseWallet>,
    pub endpoint_url: Url,
}

impl DemoAgent {
    pub async fn new(endpoint_url: Url) -> DemoAgent {
        let wallet_config = WalletConfig {
            wallet_name: format!("demo_alice_wallet_{}", uuid::Uuid::new_v4().to_string()),
            wallet_key: "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY".into(),
            wallet_key_derivation: "RAW".into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };
        let handle = create_and_open_wallet(&wallet_config).await.unwrap();
        let wallet = IndySdkWallet { wallet_handle: handle };
        DemoAgent {
            wallet: Arc::new(wallet),
            endpoint_url,
        }
    }
}
