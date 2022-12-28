use crate::api_vcx::api_global::agency_client::reset_main_agency_client;
use crate::api_vcx::api_global::pool::{close_main_pool, reset_main_pool_handle};

use crate::api_vcx::api_global::settings::get_config_value;
use crate::api_vcx::api_global::wallet::close_main_wallet;
use aries_vcx::global::settings::{
    reset_config_values, CONFIG_POOL_NAME, CONFIG_WALLET_KEY, CONFIG_WALLET_KEY_DERIVATION, CONFIG_WALLET_NAME,
    CONFIG_WALLET_TYPE, DEFAULT_POOL_NAME, DEFAULT_WALLET_NAME, UNINITIALIZED_WALLET_KEY, WALLET_KDF_DEFAULT,
};
use aries_vcx::indy::ledger::pool;
use aries_vcx::indy::wallet::{delete_wallet, WalletConfig};

pub fn state_vcx_shutdown(delete: bool) {
    info!("vcx_shutdown >>>");
    trace!("vcx_shutdown(delete: {})", delete);

    if let Ok(()) = futures::executor::block_on(close_main_wallet()) {}
    if let Ok(()) = futures::executor::block_on(close_main_pool()) {}

    crate::api_vcx::api_handle::schema::release_all();
    crate::api_vcx::api_handle::mediated_connection::release_all();
    crate::api_vcx::api_handle::issuer_credential::release_all();
    crate::api_vcx::api_handle::credential_def::release_all();
    crate::api_vcx::api_handle::proof::release_all();
    crate::api_vcx::api_handle::disclosed_proof::release_all();
    crate::api_vcx::api_handle::credential::release_all();

    if delete {
        let pool_name = get_config_value(CONFIG_POOL_NAME).unwrap_or(DEFAULT_POOL_NAME.to_string());
        let wallet_name = get_config_value(CONFIG_WALLET_NAME).unwrap_or(DEFAULT_WALLET_NAME.to_string());
        let wallet_type = get_config_value(CONFIG_WALLET_TYPE).ok();
        let wallet_key = get_config_value(CONFIG_WALLET_KEY).unwrap_or(UNINITIALIZED_WALLET_KEY.into());
        let wallet_key_derivation = get_config_value(CONFIG_WALLET_KEY_DERIVATION).unwrap_or(WALLET_KDF_DEFAULT.into());

        let _res = futures::executor::block_on(close_main_wallet());

        let wallet_config = WalletConfig {
            wallet_name,
            wallet_key,
            wallet_key_derivation,
            wallet_type,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };

        if let Ok(()) = futures::executor::block_on(delete_wallet(&wallet_config)) {}
        if let Ok(()) = futures::executor::block_on(pool::delete(&pool_name)) {}
    }

    reset_config_values();
    reset_main_agency_client();
    reset_main_pool_handle();
}
