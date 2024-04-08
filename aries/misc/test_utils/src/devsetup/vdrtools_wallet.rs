use aries_vcx_wallet::wallet::{
    base_wallet::{did_wallet::DidWallet, ManageWallet},
    indy::{indy_wallet_config::IndyWalletConfig, IndySdkWallet},
};
use log::info;

use crate::settings::{DEFAULT_WALLET_KEY, WALLET_KDF_RAW};

pub async fn dev_setup_wallet_indy(key_seed: &str) -> (String, IndySdkWallet) {
    info!("dev_setup_wallet_indy >>");
    let config_wallet = IndyWalletConfig {
        wallet_name: format!("wallet_{}", uuid::Uuid::new_v4()),
        wallet_key: DEFAULT_WALLET_KEY.into(),
        wallet_key_derivation: WALLET_KDF_RAW.into(),
        wallet_type: None,
        storage_config: None,
        storage_credentials: None,
        rekey: None,
        rekey_derivation_method: None,
    };

    let wallet = config_wallet.create_wallet().await.unwrap();

    let did_data = wallet
        .create_and_store_my_did(Some(key_seed), None)
        .await
        .unwrap();

    (did_data.did().to_owned(), wallet)
}
