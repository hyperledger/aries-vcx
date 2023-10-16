use std::{
    num::NonZeroUsize,
    sync::{Arc, RwLock},
    time::Duration,
};

use aries_vcx::{
    aries_vcx_core::{
        ledger::{
            request_submitter::vdr_ledger::{IndyVdrLedgerPool, IndyVdrSubmitter},
            response_cacher::in_memory::{
                InMemoryResponseCacherConfig, InMemoryResponseCacherConfigBuilder,
            },
        },
        wallet::base_wallet::BaseWallet,
        PoolConfig,
    },
    utils::ledger::{indyvdr_build_ledger_read, indyvdr_build_ledger_write},
};
use aries_vcx_core::ledger::{
    indy_vdr_ledger::{IndyVdrLedgerRead, IndyVdrLedgerWrite},
    response_cacher::in_memory::InMemoryResponseCacher,
};

use crate::{
    api_vcx::api_global::profile::get_main_wallet,
    errors::error::{LibvcxError, LibvcxResult},
};

pub static GLOBAL_LEDGER_INDY_READ: RwLock<
    Option<Arc<IndyVdrLedgerRead<IndyVdrSubmitter, InMemoryResponseCacher>>>,
> = RwLock::new(None);
pub static GLOBAL_LEDGER_INDY_WRITE: RwLock<Option<Arc<IndyVdrLedgerWrite<IndyVdrSubmitter>>>> =
    RwLock::new(None);

pub fn is_main_pool_open() -> bool {
    GLOBAL_LEDGER_INDY_READ
        .read()
        .map(|v| v.is_some())
        .unwrap_or_default()
}

// todo : enable opting out of caching completely be specifying 0 capacity
#[derive(Clone, Debug, Deserialize)]
// unlike internal config struct InMemoryResponseCacherConfig, this doesn't deal with Duration
// but simply numeric seconds, making it easier to pass consumers of libvcx
pub struct LibvcxInMemoryResponseCacherConfig {
    ttl_secs: NonZeroUsize,
    capacity: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LibvcxLedgerConfig {
    pub genesis_path: String,
    pub pool_config: Option<PoolConfig>,
    pub cache_config: Option<LibvcxInMemoryResponseCacherConfig>,
    pub exclude_nodes: Option<Vec<String>>,
}

impl TryFrom<LibvcxInMemoryResponseCacherConfig> for InMemoryResponseCacherConfig {
    type Error = LibvcxError;

    fn try_from(
        config: LibvcxInMemoryResponseCacherConfig,
    ) -> LibvcxResult<InMemoryResponseCacherConfig> {
        let m = InMemoryResponseCacherConfigBuilder::default()
            .ttl(Duration::from_secs(config.ttl_secs.get() as u64))
            .capacity(config.capacity)?;
        Ok(m.build())
    }
}

fn build_components_ledger(
    base_wallet: Arc<dyn BaseWallet>,
    libvcx_pool_config: &LibvcxLedgerConfig,
) -> LibvcxResult<(
    IndyVdrLedgerRead<IndyVdrSubmitter, InMemoryResponseCacher>,
    IndyVdrLedgerWrite<IndyVdrSubmitter>,
)> {
    let indy_vdr_config = match &libvcx_pool_config.pool_config {
        None => PoolConfig::default(),
        Some(cfg) => cfg.clone(),
    };
    let ledger_pool = IndyVdrLedgerPool::new(
        libvcx_pool_config.genesis_path.clone(),
        indy_vdr_config,
        libvcx_pool_config.exclude_nodes.clone().unwrap_or_default(),
    )?;
    let request_submitter = IndyVdrSubmitter::new(ledger_pool);

    let cache_config = match &libvcx_pool_config.cache_config {
        None => InMemoryResponseCacherConfig::builder()
            .ttl(Duration::from_secs(60))
            .capacity(1000)?
            .build(),
        Some(cfg) => cfg.clone().try_into()?,
    };
    let ledger_read = indyvdr_build_ledger_read(request_submitter.clone(), cache_config)?;
    let ledger_write = indyvdr_build_ledger_write(request_submitter, None);

    Ok((ledger_read, ledger_write))
}

pub fn reset_ledger_components() -> LibvcxResult<()> {
    let mut indy_read = GLOBAL_LEDGER_INDY_READ.write()?;
    *indy_read = None;
    let mut indy_write = GLOBAL_LEDGER_INDY_WRITE.write()?;
    *indy_write = None;
    Ok(())
}

pub async fn setup_ledger_components(config: &LibvcxLedgerConfig) -> LibvcxResult<()> {
    let base_wallet = get_main_wallet()?;

    let (ledger_read, ledger_write) = build_components_ledger(base_wallet, config)?;
    let mut indy_read_guard = GLOBAL_LEDGER_INDY_READ.write()?;
    *indy_read_guard = Some(Arc::new(ledger_read));
    let mut indy_write_guard = GLOBAL_LEDGER_INDY_WRITE.write()?;
    *indy_write_guard = Some(Arc::new(ledger_write));
    Ok(())
}

pub async fn open_main_pool(config: &LibvcxLedgerConfig) -> LibvcxResult<()> {
    if is_main_pool_open() {
        return Ok(());
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

    use std::num::NonZeroUsize;

    use aries_vcx::{
        aries_vcx_core::ledger::indy::pool::test_utils::{
            create_testpool_genesis_txn_file, get_temp_file_path,
        },
        global::settings::DEFAULT_GENESIS_PATH,
        utils::{
            constants::POOL1_TXN,
            devsetup::{SetupMocks, TempFile},
        },
    };
    use serde_json;

    use crate::{
        api_vcx::api_global::{
            pool::{close_main_pool, open_main_pool, reset_ledger_components, LibvcxLedgerConfig},
            profile::get_main_ledger_read,
            wallet::{close_main_wallet, test_utils::_create_and_open_wallet},
        },
        errors::error::LibvcxErrorKind,
    };

    #[test]
    fn test_deserialize_libvcx_ledger_config() {
        let data = r#"
        {
            "genesis_path": "/tmp/genesis",
            "pool_config": {
                "protocol_version": "Node1_4",
                "freshness_threshold": 300,
                "ack_timeout": 20,
                "reply_timeout": 60,
                "conn_request_limit": 5,
                "conn_active_timeout": 5,
                "request_read_nodes": 2
            },
            "cache_config": {
                "ttl_secs": 3600,
                "capacity": 1000
            }
        }
        "#;

        let config: LibvcxLedgerConfig = serde_json::from_str(data).unwrap();

        assert_eq!(config.genesis_path, "/tmp/genesis");
        assert_eq!(config.pool_config.as_ref().unwrap().protocol_version, 2);
        assert_eq!(
            config.pool_config.as_ref().unwrap().freshness_threshold,
            300
        );
        assert_eq!(config.pool_config.as_ref().unwrap().ack_timeout, 20);
        assert_eq!(config.pool_config.as_ref().unwrap().reply_timeout, 60);
        assert_eq!(config.pool_config.as_ref().unwrap().conn_request_limit, 5);
        assert_eq!(config.pool_config.as_ref().unwrap().conn_active_timeout, 5);
        assert_eq!(config.pool_config.as_ref().unwrap().request_read_nodes, 2);
        assert_eq!(
            config.cache_config.as_ref().unwrap().ttl_secs,
            NonZeroUsize::new(3600).unwrap()
        );
        assert_eq!(config.cache_config.as_ref().unwrap().capacity, 1000);
        assert!(config.exclude_nodes.is_none());
    }

    #[tokio::test]
    #[ignore]
    async fn test_open_pool() {
        let _setup = SetupMocks::init();
        _create_and_open_wallet().await.unwrap();
        let genesis_path = get_temp_file_path(DEFAULT_GENESIS_PATH)
            .to_str()
            .unwrap()
            .to_string();
        create_testpool_genesis_txn_file(&genesis_path);
        let config = LibvcxLedgerConfig {
            genesis_path,
            pool_config: None,
            cache_config: None,
            exclude_nodes: None,
        };
        open_main_pool(&config).await.unwrap();
        close_main_pool().await.unwrap();
        close_main_wallet().await.unwrap();
        reset_ledger_components().unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_open_pool_fails_if_genesis_file_is_invalid() {
        let _setup = SetupMocks::init();
        _create_and_open_wallet().await.unwrap();

        let _genesis_transactions =
            TempFile::create_with_data(POOL1_TXN, "{ \"invalid\": \"genesis\" }");

        // todo: indy-vdr panics if the file is invalid, see:
        // indy-vdr-0.3.4/src/pool/runner.rs:44:22
        assert_eq!(
            get_main_ledger_read().unwrap_err().kind(),
            LibvcxErrorKind::NotReady
        );

        close_main_wallet().await.unwrap();
        reset_ledger_components().unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_open_pool_fails_if_genesis_path_is_invalid() {
        let _setup = SetupMocks::init();
        _create_and_open_wallet().await.unwrap();

        let config = LibvcxLedgerConfig {
            genesis_path: "invalid/txn/path".to_string(),
            pool_config: None,
            cache_config: None,
            exclude_nodes: None,
        };
        assert_eq!(
            open_main_pool(&config).await.unwrap_err().kind(),
            LibvcxErrorKind::IOError
        );
        assert_eq!(
            get_main_ledger_read().unwrap_err().kind(),
            LibvcxErrorKind::NotReady
        );
        close_main_wallet().await.unwrap();
    }
}
