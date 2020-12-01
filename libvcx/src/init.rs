use futures::Future;
use indy;
use indy::ErrorCode;
use indy_sys::WalletHandle;

use crate::{settings, utils};
use crate::error::{VcxErrorExt, VcxErrorKind, VcxResult};
use crate::libindy::utils::pool::{create_pool_ledger_config, open_pool_ledger};
use crate::libindy::utils::wallet::{build_wallet_config, build_wallet_credentials, set_wallet_handle};

pub fn init_core(config: &str) -> VcxResult<()> {
    info!("init_core >>> config = {}", config);
    settings::process_config_string(&config, true)?;
    settings::log_settings();
    utils::threadpool::init();
    Ok(())
}

pub fn init_agency_client(config: &str) -> VcxResult<()> {
    info!("init_agency_client >>> config = {}", config);
    settings::get_agency_client()?.process_config_string(config, false)?;
    Ok(())
}

pub fn open_pool(pool_name: &str, path: &str, pool_config: Option<&str>) -> VcxResult<()> {
    trace!("open_pool >>> pool_name={}, path={}, pool_config={:?}", pool_name, path, pool_config);

    create_pool_ledger_config(&pool_name, &path)
        .map_err(|err| err.extend("Can not create Pool Ledger Config"))?;

    debug!("open_pool ::: Pool Config Created Successfully");

    open_pool_ledger(&pool_name, pool_config)
        .map_err(|err| err.extend("Can not open Pool Ledger"))?;

    info!("open_pool ::: Pool Opened Successfully");

    Ok(())
}

pub fn open_as_main_wallet(wallet_name: &str, wallet_key: &str, key_derivation: &str, wallet_type: Option<&str>, storage_config: Option<&str>, storage_creds: Option<&str>) -> VcxResult<WalletHandle> {
    trace!("open_as_main_wallet >>> wallet_name: {}", wallet_name);
    let config = build_wallet_config(wallet_name, wallet_type, storage_config);
    let credentials = build_wallet_credentials(wallet_key, storage_creds, key_derivation);

    let handle = indy::wallet::open_wallet(&config, &credentials)
        .wait()
        .map_err(|err|
            match err.error_code.clone() {
                ErrorCode::WalletAlreadyOpenedError => {
                    err.to_vcx(VcxErrorKind::WalletAlreadyOpen,
                               format!("Wallet \"{}\" already opened.", wallet_name))
                }
                ErrorCode::WalletAccessFailed => {
                    err.to_vcx(VcxErrorKind::WalletAccessFailed,
                               format!("Can not open wallet \"{}\". Invalid key has been provided.", wallet_name))
                }
                ErrorCode::WalletNotFoundError => {
                    err.to_vcx(VcxErrorKind::WalletNotFound,
                               format!("Wallet \"{}\" not found or unavailable", wallet_name))
                }
                error_code => {
                    err.to_vcx(VcxErrorKind::LibndyError(error_code as u32), "Indy error occurred")
                }
            })?;

    init_agency_client(&settings::settings_as_string())?;
    set_wallet_handle(handle);

    Ok(handle)
}
