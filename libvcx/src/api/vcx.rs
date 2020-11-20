use futures::future;
use indy_sys::WalletHandle;
use indy::{INVALID_WALLET_HANDLE};

use crate::error::{VcxResult, VcxError, VcxErrorKind};
use crate::service::init::{open_as_main_wallet, open_pool};
use crate::libindy::utils::wallet::{set_wallet_handle, get_wallet_handle};
use crate::{settings, utils};
use crate::utils::error;
use crate::libindy::utils::pool::{is_pool_open, create_pool_ledger_config, open_pool_ledger};

pub fn init_core(config: &str) -> VcxResult<()> {
    info!("init_core >>> config = {}", config);
    settings::process_config_string(&config, true)?;
    settings::log_settings();
    Ok(())
}

pub fn open_wallet_by_settings() -> VcxResult<WalletHandle>  {
    if get_wallet_handle() != INVALID_WALLET_HANDLE {
        error!("vcx_open_wallet :: Wallet was already initialized.");
        return Err(VcxError::from_msg(VcxErrorKind::AlreadyInitialized, "Wallet was already initialized"))
    }
    let wallet_name = match settings::get_config_value(settings::CONFIG_WALLET_NAME) {
        Ok(x) => x,
        Err(_) => {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidConfiguration, "Wallet name was not set"))
        }
    };
    let wallet_key = match settings::get_config_value(settings::CONFIG_WALLET_KEY) {
        Ok(wallet_key) => wallet_key,
        Err(_) => {
            return Err(VcxError::from_msg(VcxErrorKind::MissingWalletKey, "Wallet key was not set"))
        }
    };
    let wallet_kdf = settings::get_config_value(settings::CONFIG_WALLET_KEY_DERIVATION).unwrap_or(settings::WALLET_KDF_DEFAULT.into());
    let wallet_type = settings::get_config_value(settings::CONFIG_WALLET_TYPE).ok();
    let storage_config = settings::get_config_value(settings::CONFIG_WALLET_STORAGE_CONFIG).ok();
    let storage_creds = settings::get_config_value(settings::CONFIG_WALLET_STORAGE_CREDS).ok();

    if settings::indy_mocks_enabled() {
        set_wallet_handle(WalletHandle(1));
        info!("vcx_open_wallet :: Mocked Success");
        VcxResult::Ok(WalletHandle(1))
    } else {
        open_as_main_wallet(&wallet_name,
                            &wallet_key,
                            &wallet_kdf,
                            wallet_type.as_ref().map(String::as_str),
                            storage_config.as_ref().map(String::as_str),
                            storage_creds.as_ref().map(String::as_str))
    }
}

pub fn open_pool_by_settings() -> VcxResult<()>  {
    info!("vcx_open_pool >>>");
    if is_pool_open() {
        error!("vcx_open_pool :: Pool connection is already open.");
        return Err(VcxError::from_msg(VcxErrorKind::AlreadyInitialized, "Pool connection is already open."))
    }
    let path = match settings::get_config_value(settings::CONFIG_GENESIS_PATH) {
        Ok(result) => result,
        Err(_) => {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidConfiguration, "CONFIG_GENESIS_PATH was not configured."))
        }
    };
    let pool_name = settings::get_config_value(settings::CONFIG_POOL_NAME).unwrap_or(settings::DEFAULT_POOL_NAME.to_string());
    let pool_config = settings::get_config_value(settings::CONFIG_POOL_CONFIG).ok();
    open_pool(&pool_name, &path, pool_config.as_ref().map(String::as_str))
}

