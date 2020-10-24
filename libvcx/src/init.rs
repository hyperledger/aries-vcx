use futures::Future;
use settings;
use indy;
use indy::{ErrorCode};
use utils::libindy::pool::{create_pool_ledger_config, open_pool_ledger};
use utils::libindy::wallet::{set_wallet_handle, build_wallet_credentials, build_wallet_config};
use indy_sys::WalletHandle;
use error::{VcxErrorKind, VcxResult, VcxErrorExt};

pub fn init_core(config: &str) -> VcxResult<()> {
    info!("init_core >>> config = {}", config);
    settings::process_config_string(&config, true)?;
    settings::log_settings();
    ::utils::threadpool::init();
    Ok(())
}

pub fn open_pool(pool_name: &str, path: &str, pool_config: Option<&str>) -> VcxResult<()> {
    info!("open_pool >>> pool_name={}, path={}, pool_config={:?}", pool_name, path, pool_config);

    if settings::indy_mocks_enabled() {
        warn!("open_pool ::: Indy mocks enabled, skipping opening pool.");
        return Ok(());
    }

    trace!("open_pool ::: Opening pool {} with genesis_path: {}", pool_name, path);

    create_pool_ledger_config(&pool_name, &path)
        .map_err(|err| err.extend("Can not create Pool Ledger Config"))?;

    debug!("open_pool ::: Pool Config Created Successfully");

    open_pool_ledger(&pool_name, pool_config)
        .map_err(|err| err.extend("Can not open Pool Ledger"))?;

    info!("open_pool ::: Pool Opened Successfully");

    Ok(())
}

pub fn open_as_main_wallet(wallet_name: &str, wallet_key: &str, key_derivation: &str, wallet_type: Option<&str>, storage_config: Option<&str>, storage_creds: Option<&str>) -> VcxResult<WalletHandle> {
    if settings::indy_mocks_enabled() {
        warn!("open_as_main_wallet ::: Indy mocks enabled, skipping opening main wallet.");
        return Ok(set_wallet_handle(WalletHandle(1)));
    }

    trace!("open_wallet >>> wallet_name: {}", wallet_name);
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

    set_wallet_handle(handle);

    Ok(handle)
}