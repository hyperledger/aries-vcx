use aries_vcx_wallet::wallet::{
    askar::{askar_wallet_config::AskarWalletConfig, key_method::KeyMethod, AskarWallet},
    base_wallet::{did_wallet::DidWallet, ManageWallet},
};
use log::info;
use uuid::Uuid;

pub async fn dev_setup_wallet_askar(key_seed: &str) -> (String, AskarWallet) {
    info!("dev_setup_wallet_askar >>");
    // simple in-memory wallet
    let config_wallet = AskarWalletConfig::new(
        "sqlite://:memory:",
        KeyMethod::Unprotected,
        "",
        &Uuid::new_v4().to_string(),
    );

    let wallet = config_wallet.create_wallet().await.unwrap();

    let did_data = wallet
        .create_and_store_my_did(Some(key_seed), None)
        .await
        .unwrap();

    (did_data.did().to_owned(), wallet)
}
