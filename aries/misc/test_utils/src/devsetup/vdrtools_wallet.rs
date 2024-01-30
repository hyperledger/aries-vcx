use std::sync::Arc;

use aries_vcx_core::{
    global::settings::{DEFAULT_WALLET_KEY, WALLET_KDF_RAW},
    wallet::{
        base_wallet::{BaseWallet, ManageWallet},
        indy::wallet_config::WalletConfig,
    },
};
use log::info;

pub async fn dev_setup_wallet_indy(key_seed: &str) -> (String, Arc<dyn BaseWallet>) {
    info!("dev_setup_wallet_indy >>");
    let config_wallet = WalletConfig {
        wallet_name: format!("wallet_{}", uuid::Uuid::new_v4()),
        wallet_key: DEFAULT_WALLET_KEY.into(),
        wallet_key_derivation: WALLET_KDF_RAW.into(),
        wallet_type: None,
        storage_config: None,
        storage_credentials: None,
        rekey: None,
        rekey_derivation_method: None,
    };

    config_wallet.create_wallet().await.unwrap();
    let wallet = config_wallet.open_wallet().await.unwrap();

    let did_data = wallet
        .create_and_store_my_did(Some(key_seed), None)
        .await
        .unwrap();

    (did_data.did().to_owned(), wallet)
}
