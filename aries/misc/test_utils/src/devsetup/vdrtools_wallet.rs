use aries_vcx_core::{
    global::settings::{DEFAULT_WALLET_KEY, WALLET_KDF_RAW},
    wallet::{
        base_wallet::BaseWallet,
        indy::{
            wallet::{create_and_open_wallet, create_and_store_my_did},
            WalletConfig,
        },
    },
    WalletHandle,
};
use log::info;

pub async fn dev_setup_wallet_indy(key_seed: &str) -> (String, WalletHandle) {
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
    let wallet_handle = create_and_open_wallet(&config_wallet).await.unwrap();
    // todo: can we just extract this away? not always we end up using it (alice test agent)
    let (did, _vk) = create_and_store_my_did(wallet_handle, Some(key_seed), None)
        .await
        .unwrap();

    (did, wallet_handle)
}

pub async fn dev_build_indy_wallet(key_seed: &str) -> (String, impl BaseWallet) {
    use aries_vcx_core::wallet::indy::IndySdkWallet;

    let (public_did, wallet_handle) = dev_setup_wallet_indy(key_seed).await;
    (public_did, IndySdkWallet::new(wallet_handle))
}
