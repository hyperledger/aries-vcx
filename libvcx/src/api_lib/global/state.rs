use aries_vcx::global::settings;
use aries_vcx::indy::ledger::pool;
use aries_vcx::indy::wallet::{delete_wallet, WalletConfig};
use crate::api_lib::api_handle::vcx_settings;
use crate::api_lib::global::agency_client::reset_main_agency_client;
use crate::api_lib::global::pool::{close_main_pool, reset_main_pool_handle};
use crate::api_lib::global::wallet::close_main_wallet;

pub fn state_vcx_shutdown(delete: bool) {
    info!("vcx_shutdown >>>");
    trace!("vcx_shutdown(delete: {})", delete);

    match futures::executor::block_on(close_main_wallet()) {
        Ok(()) => {}
        Err(_) => {}
    };

    match futures::executor::block_on(close_main_pool()) {
        Ok(()) => {}
        Err(_) => {}
    };

    crate::api_lib::api_handle::schema::release_all();
    crate::api_lib::api_handle::mediated_connection::release_all();
    crate::api_lib::api_handle::issuer_credential::release_all();
    crate::api_lib::api_handle::credential_def::release_all();
    crate::api_lib::api_handle::proof::release_all();
    crate::api_lib::api_handle::disclosed_proof::release_all();
    crate::api_lib::api_handle::credential::release_all();

    if delete {
        let pool_name =
            vcx_settings::get_config_value(settings::CONFIG_POOL_NAME).unwrap_or(settings::DEFAULT_POOL_NAME.to_string());
        let wallet_name = vcx_settings::get_config_value(settings::CONFIG_WALLET_NAME)
            .unwrap_or(settings::DEFAULT_WALLET_NAME.to_string());
        let wallet_type = vcx_settings::get_config_value(settings::CONFIG_WALLET_TYPE).ok();
        let wallet_key = vcx_settings::get_config_value(settings::CONFIG_WALLET_KEY)
            .unwrap_or(settings::UNINITIALIZED_WALLET_KEY.into());
        let wallet_key_derivation = vcx_settings::get_config_value(settings::CONFIG_WALLET_KEY_DERIVATION)
            .unwrap_or(settings::WALLET_KDF_DEFAULT.into());

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

        match futures::executor::block_on(delete_wallet(&wallet_config)) {
            Ok(()) => (),
            Err(_) => (),
        };

        match futures::executor::block_on(pool::delete(&pool_name)) {
            Ok(()) => (),
            Err(_) => (),
        };
    }

    settings::reset_config_values();
    reset_main_agency_client();
    reset_main_pool_handle();
}
