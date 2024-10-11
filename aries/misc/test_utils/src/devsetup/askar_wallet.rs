use aries_vcx_wallet::wallet::{
    askar::{askar_wallet_config::AskarWalletConfig, key_method::KeyMethod, AskarWallet},
    base_wallet::{did_wallet::DidWallet, ManageWallet},
};
use log::info;
use uuid::Uuid;


pub async fn dev_setup_wallet_askar(key_seed: &str) -> (String, AskarWallet) {
    info!("dev_setup_wallet_askar >>");
    // TODO - actually impl this
    let config_wallet = AskarWalletConfig::new(
        "sqlite://:memory:",
        KeyMethod::Unprotected,
        "",
        &Uuid::new_v4().to_string(),
    );
    // wallet_name: format!("wallet_{}", uuid::Uuid::new_v4()),
    // wallet_key: DEFAULT_WALLET_KEY.into(),
    // wallet_key_derivation: WALLET_KDF_RAW.into(),
    // wallet_type: None,
    // storage_config: None,
    // storage_credentials: None,
    // rekey: None,
    // rekey_derivation_method: None,

    let wallet = config_wallet.create_wallet().await.unwrap();

    let did_data = wallet
        .create_and_store_my_did(Some(key_seed), None)
        .await
        .unwrap();

    (did_data.did().to_owned(), wallet)
}
