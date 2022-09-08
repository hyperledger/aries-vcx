use std::sync::RwLock;

use crate::error::{VcxError, VcxErrorKind, VcxResult};
use crate::global::settings;
use crate::libindy::utils::pool::PoolConfig;
use crate::libindy::utils::pool::{create_pool_ledger_config, open_pool_ledger, close, delete};

lazy_static! {
    static ref POOL_HANDLE: RwLock<Option<i32>> = RwLock::new(None);
}

pub fn set_main_pool_handle(handle: Option<i32>) {
    let mut h = POOL_HANDLE.write().unwrap();
    *h = handle;
}

pub fn get_main_pool_handle() -> VcxResult<i32> {
    POOL_HANDLE
        .read()
        .or(Err(VcxError::from_msg(
            VcxErrorKind::NoPoolOpen,
            "There is no pool opened",
        )))?
        .ok_or(VcxError::from_msg(VcxErrorKind::NoPoolOpen, "There is no pool opened"))
}

pub fn is_main_pool_open() -> bool {
    get_main_pool_handle().is_ok()
}

pub fn reset_main_pool_handle() {
    set_main_pool_handle(None);
}

pub async fn open_main_pool(config: &PoolConfig) -> VcxResult<()> {
    let pool_name = config
        .pool_name
        .clone()
        .unwrap_or(settings::DEFAULT_POOL_NAME.to_string());
    trace!(
        "open_pool >>> pool_name: {}, path: {}, pool_config: {:?}",
        pool_name,
        config.genesis_path,
        config.pool_config
    );

    create_pool_ledger_config(&pool_name, &config.genesis_path)
        .await
        .map_err(|err| err.extend("Can not create Pool Ledger Config"))?;

    debug!("open_pool ::: Pool Config Created Successfully");

    let handle = open_pool_ledger(&pool_name, config.pool_config.as_deref())
        .await
        .map_err(|err| err.extend("Can not open Pool Ledger"))?;

    set_main_pool_handle(Some(handle));

    info!("open_pool ::: Pool Opened Successfully");

    Ok(())
}

pub async fn close_main_pool() -> VcxResult<()> {
    info!("close_main_pool ::: Closing main pool");
    close(get_main_pool_handle()?).await?;
    Ok(())
}
