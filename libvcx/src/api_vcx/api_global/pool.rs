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
    let mut h = POOL_HANDLE.write().expect("Unable to access POOL_HANDLE");
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

#[cfg(test)]
pub mod tests {
    use crate::api_c::vcx::vcx_open_main_pool;
    use crate::api_vcx::api_global::pool::{get_main_pool_handle, open_main_pool, reset_main_pool_handle};
    use crate::errors::error::LibvcxErrorKind;
    use aries_vcx::global::settings::{set_config_value, CONFIG_GENESIS_PATH};
    use aries_vcx::indy::ledger::pool::test_utils::{
        create_tmp_genesis_txn_file, delete_named_test_pool, delete_test_pool,
    };
    use aries_vcx::indy::ledger::pool::PoolConfig;
    use aries_vcx::utils::constants::GENESIS_PATH;
    use aries_vcx::utils::devsetup::{SetupDefaults, SetupEmpty, TempFile};

    #[tokio::test]
    #[cfg(feature = "pool_tests")]
    async fn test_open_pool() {
        let _setup = SetupEmpty::init();

        let genesis_path = create_tmp_genesis_txn_file();
        let config = PoolConfig {
            genesis_path,
            pool_name: None,
            pool_config: None,
        };
        open_main_pool(&config).await.unwrap();
        delete_test_pool(get_main_pool_handle().unwrap()).await;
        reset_main_pool_handle();
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_open_pool_fails_if_genesis_file_is_invalid() {
        let _setup = SetupEmpty::init();
        let pool_name = format!("invalidpool_{}", uuid::Uuid::new_v4().to_string());

        // Write invalid genesis.txn
        let _genesis_transactions = TempFile::create_with_data(GENESIS_PATH, "{ \"invalid\": \"genesis\" }");

        set_config_value(CONFIG_GENESIS_PATH, &_genesis_transactions.path).unwrap();

        let pool_config = PoolConfig {
            genesis_path: _genesis_transactions.path.clone(),
            pool_name: Some(pool_name.clone()),
            pool_config: None,
        };
        // let err = _vcx_open_main_pool_c_closure(&json!(pool_config).to_string()).unwrap_err();
        assert_eq!(
            open_main_pool(&pool_config).await.unwrap_err().kind(),
            LibvcxErrorKind::PoolLedgerConnect
        );
        assert_eq!(get_main_pool_handle().unwrap_err().kind(), LibvcxErrorKind::NoPoolOpen);

        delete_named_test_pool(0, &pool_name);
        reset_main_pool_handle();
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_open_pool_fails_if_genesis_path_is_invalid() {
        let _setup = SetupDefaults::init();
        let pool_name = format!("invalidpool_{}", uuid::Uuid::new_v4().to_string());

        let pool_config = PoolConfig {
            genesis_path: "invalid/txn/path".to_string(),
            pool_name: Some(pool_name.clone()),
            pool_config: None,
        };
        assert_eq!(
            open_main_pool(&pool_config).await.unwrap_err().kind(),
            LibvcxErrorKind::InvalidGenesisTxnPath
        );
        assert_eq!(get_main_pool_handle().unwrap_err().kind(), LibvcxErrorKind::NoPoolOpen);
    }
}
