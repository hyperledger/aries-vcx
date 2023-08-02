use aries_vcx::aries_vcx_core::ledger::base_ledger::{
    AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite, TaaConfigurator,
};
use std::num::NonZeroUsize;

use crate::api_vcx::api_global::profile::get_main_wallet;
use crate::errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult};
use aries_vcx::aries_vcx_core::ledger::request_submitter::vdr_ledger::{IndyVdrLedgerPool, IndyVdrSubmitter};
use aries_vcx::aries_vcx_core::ledger::response_cacher::in_memory::InMemoryResponseCacherConfig;
use aries_vcx::aries_vcx_core::wallet::base_wallet::BaseWallet;
use aries_vcx::aries_vcx_core::PoolConfig;
use aries_vcx::core::profile::ledger::indyvdr_build_ledger_read;
use aries_vcx::core::profile::ledger::indyvdr_build_ledger_write;
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::errors::error::VcxResult;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

lazy_static! {
    pub static ref global_ledger_anoncreds_read: RwLock<Option<Arc<dyn AnoncredsLedgerRead>>> = RwLock::new(None);
    pub static ref global_ledger_anoncreds_write: RwLock<Option<Arc<dyn AnoncredsLedgerWrite>>> = RwLock::new(None);
    pub static ref global_ledger_indy_read: RwLock<Option<Arc<dyn IndyLedgerRead>>> = RwLock::new(None);
    pub static ref global_ledger_indy_write: RwLock<Option<Arc<dyn IndyLedgerWrite>>> = RwLock::new(None);
    pub static ref global_taa_configurator: RwLock<Option<Arc<dyn TaaConfigurator>>> = RwLock::new(None);
}

pub fn is_main_pool_open() -> bool {
    false
    // todo: implement this, based on whether ledger read is Some or None
    // global_profile.inject_anoncreds_ledger_read()
}

// todo : enable opting out of caching completely be specifying 0 capacity
#[derive(Clone, Debug, Deserialize)]
// unlike internal config struct InMemoryResponseCacherConfig, this doesn't deal with Duration
// but simply numeric seconds, making it easier to pass consumers of libvcx
pub struct LibvcxInMemoryResponseCacherConfig {
    ttl_secs: NonZeroUsize,
    capacity: NonZeroUsize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LibvcxLedgerConfig {
    pub genesis_path: String,
    pub pool_config: Option<PoolConfig>,
    pub cache_config: Option<LibvcxInMemoryResponseCacherConfig>,
}

impl From<LibvcxInMemoryResponseCacherConfig> for InMemoryResponseCacherConfig {
    fn from(config: LibvcxInMemoryResponseCacherConfig) -> Self {
        InMemoryResponseCacherConfig {
            ttl: Duration::from_secs(config.ttl_secs.get() as u64),
            capacity: config.capacity,
        }
    }
}

async fn build_components_ledger(
    base_wallet: Arc<dyn BaseWallet>,
    libvcx_pool_config: &LibvcxLedgerConfig,
) -> VcxResult<(
    Arc<dyn AnoncredsLedgerRead>,
    Arc<dyn AnoncredsLedgerWrite>,
    Arc<dyn IndyLedgerRead>,
    Arc<dyn IndyLedgerWrite>,
    Arc<dyn TaaConfigurator>,
)> {
    let cache_config = match &libvcx_pool_config.cache_config {
        None => InMemoryResponseCacherConfig::builder()
            .ttl(Duration::from_secs(60))
            .capacity(1000)?
            .build(),
        Some(cfg) => cfg.clone().into(),
    };
    let indy_vdr_config = match &libvcx_pool_config.pool_config {
        None => PoolConfig::default(),
        Some(cfg) => cfg.clone(),
    };
    let ledger_pool = Arc::new(IndyVdrLedgerPool::new(
        libvcx_pool_config.genesis_path.clone(),
        indy_vdr_config,
    )?);
    let request_submitter = Arc::new(IndyVdrSubmitter::new(ledger_pool));

    let ledger_read = Arc::new(indyvdr_build_ledger_read(request_submitter.clone(), cache_config)?);
    let ledger_write = Arc::new(indyvdr_build_ledger_write(base_wallet, request_submitter, None));
    let taa_configurator: Arc<dyn TaaConfigurator> = ledger_write.clone();
    let anoncreds_write: Arc<dyn AnoncredsLedgerWrite> = ledger_write.clone();
    let indy_write: Arc<dyn IndyLedgerWrite> = ledger_write.clone();
    let anoncreds_read: Arc<dyn AnoncredsLedgerRead> = ledger_read.clone();
    let indy_read: Arc<dyn IndyLedgerRead> = ledger_read.clone();
    return Ok((anoncreds_read, anoncreds_write, indy_read, indy_write, taa_configurator));
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
    let mut taa_configurator = global_taa_configurator.write()?;
    *taa_configurator = None;
    Ok(())
}

pub async fn setup_ledger_components(config: &LibvcxLedgerConfig) -> LibvcxResult<()> {
    let base_wallet = get_main_wallet()?;
    let (anoncreds_read, anoncreds_write, indy_read, indy_write, taa_configurator) =
        build_components_ledger(base_wallet, config).await?;
    let mut anoncreds_read_guard = global_ledger_anoncreds_read.write()?;
    *anoncreds_read_guard = Some(anoncreds_read.clone());
    let mut anoncreds_write_guard = global_ledger_anoncreds_write.write()?;
    *anoncreds_write_guard = Some(anoncreds_write.clone());
    let mut indy_read_guard = global_ledger_indy_read.write()?;
    *indy_read_guard = Some(indy_read.clone());
    let mut indy_write_guard = global_ledger_indy_write.write()?;
    *indy_write_guard = Some(indy_write.clone());
    let mut indy_taa_configurator = global_taa_configurator.write()?;
    *indy_taa_configurator = Some(taa_configurator.clone());
    Ok(())
}

pub async fn open_main_pool(config: &LibvcxLedgerConfig) -> LibvcxResult<()> {
    if is_main_pool_open() {
        error!("open_main_pool >> Pool connection is already open.");
        return Err(LibvcxError::from_msg(
            LibvcxErrorKind::AlreadyInitialized,
            "Pool connection is already open.",
        ));
    }

    trace!(
        "open_pool >> path: {}, pool_config: {:?}",
        config.genesis_path,
        config.pool_config
    );

    setup_ledger_components(config).await?;

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
    use crate::api_vcx::api_global::pool::{
        close_main_pool, open_main_pool, reset_ledger_components, LibvcxLedgerConfig,
    };
    use crate::api_vcx::api_global::profile::get_main_anoncreds_ledger_read;
    use crate::api_vcx::api_global::wallet::close_main_wallet;
    use crate::api_vcx::api_global::wallet::test_utils::_create_and_open_wallet;
    use crate::errors::error::LibvcxErrorKind;
    use aries_vcx::aries_vcx_core::ledger::indy::pool::test_utils::{
        create_testpool_genesis_txn_file, get_temp_file_path,
    };
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
        let config = LibvcxLedgerConfig {
            genesis_path,
            pool_config: None,
            cache_config: None,
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

        let genesis_transactions = TempFile::create_with_data(POOL1_TXN, "{ \"invalid\": \"genesis\" }");
        set_config_value(CONFIG_GENESIS_PATH, &genesis_transactions.path).unwrap();
        let config = LibvcxLedgerConfig {
            genesis_path: genesis_transactions.path.clone(),
            pool_config: None,
            cache_config: None,
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

        let config = LibvcxLedgerConfig {
            genesis_path: "invalid/txn/path".to_string(),
            pool_config: None,
            cache_config: None,
        };
        assert_eq!(
            open_main_pool(&config).await.unwrap_err().kind(),
            LibvcxErrorKind::IOError
        );
        assert_eq!(
            get_main_anoncreds_ledger_read().unwrap_err().kind(),
            LibvcxErrorKind::NotReady
        );
        close_main_wallet().await.unwrap();
    }
}
