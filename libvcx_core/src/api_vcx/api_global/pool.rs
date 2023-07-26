use aries_vcx::aries_vcx_core::ledger::base_ledger::{
    AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite,
};
use aries_vcx::global::settings::{indy_mocks_enabled, DEFAULT_POOL_NAME};

use crate::api_vcx::api_global::profile::get_main_wallet;
use crate::api_vcx::api_global::wallet::get_main_wallet_handle;
use crate::errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult};
use aries_vcx::aries_vcx_core::errors::error::AriesVcxCoreError;
use aries_vcx::aries_vcx_core::ledger::indy::pool::PoolConfig;
use aries_vcx::aries_vcx_core::ledger::request_submitter::vdr_ledger::{
    IndyVdrLedgerPool, IndyVdrSubmitter, LedgerPoolConfig,
};
use aries_vcx::aries_vcx_core::wallet::base_wallet::BaseWallet;
use aries_vcx::core::profile::ledger::indyvdr_build_ledger_read;
use aries_vcx::core::profile::ledger::indyvdr_build_ledger_write;
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::errors::error::{AriesVcxError, VcxResult};
use std::sync::Arc;
use std::sync::RwLock;

lazy_static! {
    pub static ref global_ledger_anoncreds_read: RwLock<Option<Arc<dyn AnoncredsLedgerRead>>> = RwLock::new(None);
    pub static ref global_ledger_anoncreds_write: RwLock<Option<Arc<dyn AnoncredsLedgerWrite>>> = RwLock::new(None);
    pub static ref global_ledger_indy_read: RwLock<Option<Arc<dyn IndyLedgerRead>>> = RwLock::new(None);
    pub static ref global_ledger_indy_write: RwLock<Option<Arc<dyn IndyLedgerWrite>>> = RwLock::new(None);
}

pub fn is_main_pool_open() -> bool {
    false
    // todo: implement this, based on whether ledger read is Some or None
    // global_profile.inject_anoncreds_ledger_read()
}

async fn build_components_ledger(
    base_wallet: Arc<dyn BaseWallet>,
    pool_name: String,
    config: &PoolConfig,
) -> VcxResult<(
    Arc<dyn AnoncredsLedgerRead>,
    Arc<dyn AnoncredsLedgerWrite>,
    Arc<dyn IndyLedgerRead>,
    Arc<dyn IndyLedgerWrite>,
)> {
        let ledger_pool_config = LedgerPoolConfig {
            genesis_file_path: config.genesis_path.clone(),
        };
        let ledger_pool = Arc::new(IndyVdrLedgerPool::new(ledger_pool_config)?);
        let request_submitter = Arc::new(IndyVdrSubmitter::new(ledger_pool));

        let ledger_read = Arc::new(indyvdr_build_ledger_read(request_submitter.clone())?);
        let ledger_write = Arc::new(indyvdr_build_ledger_write(base_wallet, request_submitter, None));
        let anoncreds_read: Arc<dyn AnoncredsLedgerRead> = ledger_read.clone();
        let anoncreds_write: Arc<dyn AnoncredsLedgerWrite> = ledger_write.clone();
        let indy_read: Arc<dyn IndyLedgerRead> = ledger_read.clone();
        let indy_write: Arc<dyn IndyLedgerWrite> = ledger_write.clone();
        return Ok((anoncreds_read, anoncreds_write, indy_read, indy_write));
}

pub fn reset_ledger_components() -> LibvcxResult<()> {
    let mut anoncreds_read = global_ledger_anoncreds_read.write()?;
    *anoncreds_read = None;
    let mut anoncreds_write = global_ledger_anoncreds_write.write()?;
    *anoncreds_write = None;
    let mut indy_read = global_ledger_indy_read.write()?;
    *indy_read = None;
    let mut indy_write = global_ledger_indy_write.write()?;
    *indy_write = None;
    Ok(())
}

pub async fn setup_ledger_components(pool_name: String, config: &PoolConfig) -> LibvcxResult<()> {
    let base_wallet = get_main_wallet()?;
    let (anoncreds_read, anoncreds_write, indy_read, indy_write) =
        build_components_ledger(base_wallet, pool_name, config).await?;
    let mut anoncreds_read_guard = global_ledger_anoncreds_read.write()?;
    *anoncreds_read_guard = Some(anoncreds_read.clone() as Arc<dyn AnoncredsLedgerRead>);
    let mut anoncreds_write_guard = global_ledger_anoncreds_write.write()?;
    *anoncreds_write_guard = Some(anoncreds_write.clone() as Arc<dyn AnoncredsLedgerWrite>);
    let mut indy_read_guard = global_ledger_indy_read.write()?;
    *indy_read_guard = Some(indy_read.clone() as Arc<dyn IndyLedgerRead>);
    let mut indy_write_guard = global_ledger_indy_write.write()?;
    *indy_write_guard = Some(indy_write.clone() as Arc<dyn IndyLedgerWrite>);
    Ok(())
}

// todo: seems like we need to bring back PoolConfig, which was input for libvcx, and we share that for
//       indy-vdr implementation regardless currently
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

    setup_ledger_components(pool_name, config).await?;

    info!("open_pool >> Pool Opened Successfully");

    Ok(())
}

pub async fn close_main_pool() -> LibvcxResult<()> {
    info!("close_main_pool >> Closing main pool");

    reset_ledger_components()?;
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use crate::api_vcx::api_global::pool::{close_main_pool, open_main_pool, reset_ledger_components};
    use crate::api_vcx::api_global::profile::get_main_anoncreds_ledger_read;
    use crate::api_vcx::api_global::wallet::close_main_wallet;
    use crate::api_vcx::api_global::wallet::test_utils::_create_and_open_wallet;
    use crate::errors::error::LibvcxErrorKind;
    use aries_vcx::aries_vcx_core::ledger::indy::pool::test_utils::{
        create_testpool_genesis_txn_file, get_temp_file_path,
    };
    use aries_vcx::aries_vcx_core::ledger::indy::pool::PoolConfig;
    use aries_vcx::core::profile::profile::Profile;
    use aries_vcx::global::settings::{set_config_value, CONFIG_GENESIS_PATH, DEFAULT_GENESIS_PATH};
    use aries_vcx::utils::constants::POOL1_TXN;
    use aries_vcx::utils::devsetup::{SetupDefaults, SetupEmpty, TempFile};

    #[tokio::test]
    #[ignore]
    async fn test_open_pool() {
        let _setup = SetupEmpty::init();
        _create_and_open_wallet().await.unwrap();
        let genesis_path = get_temp_file_path(DEFAULT_GENESIS_PATH).to_str().unwrap().to_string();
        create_testpool_genesis_txn_file(&genesis_path);
        let config = PoolConfig {
            genesis_path,
            pool_name: None,
            pool_config: None,
        };
        open_main_pool(&config).await.unwrap();
        close_main_pool().await.unwrap();
        close_main_wallet().await.unwrap();
        reset_ledger_components().unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_open_pool_fails_if_genesis_file_is_invalid() {
        let _setup = SetupEmpty::init();
        _create_and_open_wallet().await.unwrap();
        let pool_name = format!("invalidpool_{}", uuid::Uuid::new_v4().to_string());

        // Write invalid genesis.txn
        let _genesis_transactions = TempFile::create_with_data(POOL1_TXN, "{ \"invalid\": \"genesis\" }");

        set_config_value(CONFIG_GENESIS_PATH, &_genesis_transactions.path).unwrap();

        let pool_config = PoolConfig {
            genesis_path: _genesis_transactions.path.clone(),
            pool_name: Some(pool_name.clone()),
            pool_config: None,
        };
        // todo: indy-vdr panics if the file is invalid, see: indy-vdr-0.3.4/src/pool/runner.rs:44:22
        assert_eq!(
            get_main_anoncreds_ledger_read().unwrap_err().kind(),
            LibvcxErrorKind::NotReady
        );

        close_main_wallet().await.unwrap();
        reset_ledger_components().unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_open_pool_fails_if_genesis_path_is_invalid() {
        let _setup = SetupDefaults::init();
        _create_and_open_wallet().await.unwrap();
        let pool_name = format!("invalidpool_{}", uuid::Uuid::new_v4().to_string());

        let pool_config = PoolConfig {
            genesis_path: "invalid/txn/path".to_string(),
            pool_name: Some(pool_name.clone()),
            pool_config: None,
        };
        assert_eq!(
            open_main_pool(&pool_config).await.unwrap_err().kind(),
            LibvcxErrorKind::IOError
        );
        assert_eq!(
            get_main_anoncreds_ledger_read().unwrap_err().kind(),
            LibvcxErrorKind::NotReady
        );
        close_main_wallet().await.unwrap();
    }
}
