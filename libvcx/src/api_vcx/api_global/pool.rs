use aries_vcx::global::settings::{indy_mocks_enabled, DEFAULT_POOL_NAME};
use std::sync::RwLock;

use crate::errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult};
use aries_vcx::indy::ledger::pool::PoolConfig;
use aries_vcx::indy::ledger::pool::{close, create_pool_ledger_config, open_pool_ledger};
use aries_vcx::vdrtools::INVALID_POOL_HANDLE;

lazy_static! {
    static ref POOL_HANDLE: RwLock<Option<i32>> = RwLock::new(None);
}

pub fn set_main_pool_handle(handle: Option<i32>) {
    let mut h = POOL_HANDLE.write().unwrap();
    *h = handle;
}

pub fn get_main_pool_handle() -> LibvcxResult<i32> {
    if indy_mocks_enabled() {
        return Ok(INVALID_POOL_HANDLE);
    }
    POOL_HANDLE
        .read()
        .or(Err(LibvcxError::from_msg(
            LibvcxErrorKind::NoPoolOpen,
            "There is no pool opened",
        )))?
        .ok_or(LibvcxError::from_msg(
            LibvcxErrorKind::NoPoolOpen,
            "There is no pool opened",
        ))
}

pub fn is_main_pool_open() -> bool {
    get_main_pool_handle().is_ok()
}

pub fn reset_main_pool_handle() {
    set_main_pool_handle(None);
}

pub async fn open_main_pool(config: &PoolConfig) -> LibvcxResult<()> {
    if is_main_pool_open() {
        error!("vcx_open_main_pool :: Pool connection is already open.");
        return Err(LibvcxError::from_msg(
            LibvcxErrorKind::AlreadyInitialized,
            "Pool connection is already open.",
        ));
    }

    let pool_name = config.pool_name.clone().unwrap_or(DEFAULT_POOL_NAME.to_string());
    trace!(
        "open_pool >>> pool_name: {}, path: {}, pool_config: {:?}",
        pool_name,
        config.genesis_path,
        config.pool_config
    );

    create_pool_ledger_config(&pool_name, &config.genesis_path)
        .map_err(|err| err.extend("Can not create Pool Ledger Config"))?;

    debug!("open_pool ::: Pool Config Created Successfully");

    let handle = open_pool_ledger(&pool_name, Some(config.clone()))
        .await
        .map_err(|err| err.extend("Can not open Pool Ledger"))?;

    set_main_pool_handle(Some(handle));

    info!("open_pool ::: Pool Opened Successfully");

    Ok(())
}

pub async fn close_main_pool() -> LibvcxResult<()> {
    info!("close_main_pool ::: Closing main pool");
    close(get_main_pool_handle()?).await?;
    Ok(())
}
