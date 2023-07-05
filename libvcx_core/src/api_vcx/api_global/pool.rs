use aries_vcx::aries_vcx_core::indy::ledger::pool::{
    create_pool_ledger_config, indy_close_pool, indy_open_pool, PoolConfig,
};
use aries_vcx::aries_vcx_core::ledger::base_ledger::{
    AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite,
};
use aries_vcx::aries_vcx_core::{PoolHandle, INVALID_POOL_HANDLE};
use aries_vcx::global::settings::{indy_mocks_enabled, DEFAULT_POOL_NAME};

use crate::api_vcx::api_global::wallet::get_main_wallet_handle;
use crate::errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult};
use aries_vcx::aries_vcx_core::ledger::indy_ledger::{IndySdkLedgerRead, IndySdkLedgerWrite};
use aries_vcx::core::profile::profile::Profile;
use std::sync::Arc;
use std::sync::RwLock;

lazy_static! {
    static ref POOL_HANDLE: RwLock<Option<i32>> = RwLock::new(None);
}

pub fn set_vdrtools_global_pool_handle(handle: Option<i32>) {
    let mut h = POOL_HANDLE.write().expect("Unable to access POOL_HANDLE");
    *h = handle;
}

pub fn get_vdrtools_global_pool_handle() -> LibvcxResult<i32> {
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

lazy_static! {
    pub static ref ledger_anoncreds_read: RwLock<Option<Arc<dyn AnoncredsLedgerRead>>> = RwLock::new(None);
    pub static ref ledger_anoncreds_write: RwLock<Option<Arc<dyn AnoncredsLedgerWrite>>> = RwLock::new(None);
    pub static ref ledger_indy_read: RwLock<Option<Arc<dyn IndyLedgerRead>>> = RwLock::new(None);
    pub static ref ledger_indy_write: RwLock<Option<Arc<dyn IndyLedgerWrite>>> = RwLock::new(None);
}

pub fn is_main_pool_open() -> bool {
    false
    // todo: implement this, based on whether ledger read is Some or None
    // global_profile.inject_anoncreds_ledger_read()
}

pub fn reset_global_ledger_components() -> LibvcxResult<()> {
    setup_ledger_components(None)?;
    set_vdrtools_global_pool_handle(None);
    Ok(())
}

pub fn setup_ledger_components(handle: Option<PoolHandle>) -> LibvcxResult<()> {
    match handle {
        None => {
            let mut anoncreds_read = ledger_anoncreds_read.write()?;
            *anoncreds_read = None;
            let mut anoncreds_write = ledger_anoncreds_write.write()?;
            *anoncreds_write = None;
            let mut indy_read = ledger_indy_read.write()?;
            *indy_read = None;
            let mut indy_write = ledger_indy_write.write()?;
            *indy_write = None;
        }
        Some(pool_handle) => {
            let wallet_handle = get_main_wallet_handle()?;
            let ledger_read = Arc::new(IndySdkLedgerRead::new(wallet_handle, pool_handle));
            let ledger_write = Arc::new(IndySdkLedgerWrite::new(wallet_handle, pool_handle));
            let mut anoncreds_read = ledger_anoncreds_read.write()?;
            *anoncreds_read = Some(ledger_read.clone() as Arc<dyn AnoncredsLedgerRead>);
            let mut anoncreds_write = ledger_anoncreds_write.write()?;
            *anoncreds_write = Some(ledger_write.clone() as Arc<dyn AnoncredsLedgerWrite>);
            let mut indy_read = ledger_indy_read.write()?;
            *indy_read = Some(ledger_read.clone() as Arc<dyn IndyLedgerRead>);
            let mut indy_write = ledger_indy_write.write()?;
            *indy_write = Some(ledger_write.clone() as Arc<dyn IndyLedgerWrite>);
        }
    }
    Ok(())
}

pub async fn open_main_pool(config: &PoolConfig) -> LibvcxResult<()> {
    if is_main_pool_open() {
        error!("open_main_pool >> Pool connection is already open.");
        return Err(LibvcxError::from_msg(
            LibvcxErrorKind::AlreadyInitialized,
            "Pool connection is already open.",
        ));
    }

    let pool_name = config.pool_name.clone().unwrap_or(DEFAULT_POOL_NAME.to_string());
    trace!(
        "open_pool >> pool_name: {}, path: {}, pool_config: {:?}",
        pool_name,
        config.genesis_path,
        config.pool_config
    );

    create_pool_ledger_config(&pool_name, &config.genesis_path)
        .map_err(|err| err.extend("Can not create Pool Ledger Config"))?;

    debug!("open_pool >> Pool Config Created Successfully");

    let pool_handle = indy_open_pool(&pool_name, config.pool_config.clone())
        .await
        .map_err(|err| err.extend("Can not open Pool Ledger"))?;

    set_vdrtools_global_pool_handle(Some(pool_handle));
    setup_ledger_components(Some(pool_handle))?;

    info!("open_pool >> Pool Opened Successfully");

    Ok(())
}

pub async fn close_main_pool() -> LibvcxResult<()> {
    info!("close_main_pool >> Closing main pool");
    indy_close_pool(get_vdrtools_global_pool_handle()?).await?;
    // todo: better way to go about this?
    set_vdrtools_global_pool_handle(None);
    setup_ledger_components(None)?;
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use crate::api_vcx::api_global::pool::{close_main_pool, open_main_pool, reset_global_ledger_components};
    use crate::api_vcx::api_global::profile::get_main_anoncreds_ledger_read;
    use crate::api_vcx::api_global::wallet::test_utils::_create_and_open_wallet;
    use crate::errors::error::LibvcxErrorKind;
    use aries_vcx::aries_vcx_core::indy::ledger::pool::test_utils::create_testpool_genesis_txn_file;
    use aries_vcx::aries_vcx_core::indy::ledger::pool::{indy_delete_pool, PoolConfig};
    use aries_vcx::aries_vcx_core::INVALID_POOL_HANDLE;
    use aries_vcx::core::profile::profile::Profile;
    use aries_vcx::global::settings::{set_config_value, CONFIG_GENESIS_PATH, DEFAULT_GENESIS_PATH};
    use aries_vcx::utils::constants::POOL1_TXN;
    use aries_vcx::utils::devsetup::{SetupDefaults, SetupEmpty, TempFile};
    use aries_vcx::utils::get_temp_dir_path;

    #[tokio::test]
    #[ignore]
    async fn test_open_pool() {
        let _setup = SetupEmpty::init();
        _create_and_open_wallet().await.unwrap();
        let genesis_path = get_temp_dir_path(DEFAULT_GENESIS_PATH).to_str().unwrap().to_string();
        create_testpool_genesis_txn_file(&genesis_path);
        let config = PoolConfig {
            genesis_path,
            pool_name: None,
            pool_config: None,
        };
        open_main_pool(&config).await.unwrap();
        close_main_pool().await.unwrap();
        reset_global_ledger_components().unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_open_pool_fails_if_genesis_file_is_invalid() {
        let _setup = SetupEmpty::init();
        let pool_name = format!("invalidpool_{}", uuid::Uuid::new_v4().to_string());

        // Write invalid genesis.txn
        let _genesis_transactions = TempFile::create_with_data(POOL1_TXN, "{ \"invalid\": \"genesis\" }");

        set_config_value(CONFIG_GENESIS_PATH, &_genesis_transactions.path).unwrap();

        let pool_config = PoolConfig {
            genesis_path: _genesis_transactions.path.clone(),
            pool_name: Some(pool_name.clone()),
            pool_config: None,
        };
        assert_eq!(
            open_main_pool(&pool_config).await.unwrap_err().kind(),
            LibvcxErrorKind::PoolLedgerConnect
        );
        assert_eq!(
            get_main_anoncreds_ledger_read().unwrap_err().kind(),
            LibvcxErrorKind::NotReady
        );

        indy_delete_pool(&pool_name).await.unwrap();
        reset_global_ledger_components().unwrap();
    }

    #[tokio::test]
    #[ignore]
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
        assert_eq!(
            get_main_anoncreds_ledger_read().unwrap_err().kind(),
            LibvcxErrorKind::NotReady
        );
    }
}
