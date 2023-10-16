#![allow(clippy::unwrap_used)]

use std::{
    env, fs,
    future::Future,
    sync::{Arc, Once},
};

use agency_client::testing::mocking::{enable_agency_mocks, AgencyMockDecrypted};
use aries_vcx_core::{
    anoncreds::{base_anoncreds::BaseAnonCreds, credx_anoncreds::IndyCredxAnonCreds},
    ledger::{
        base_ledger::{
            AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite,
            TxnAuthrAgrmtOptions,
        },
        indy::pool::test_utils::{create_testpool_genesis_txn_file, get_temp_file_path},
        indy_vdr_ledger::{
            GetTxnAuthorAgreementData, IndyVdrLedgerRead, IndyVdrLedgerReadConfig,
            IndyVdrLedgerWrite, IndyVdrLedgerWriteConfig, ProtocolVersion,
        },
        request_submitter::{
            vdr_ledger::{IndyVdrLedgerPool, IndyVdrSubmitter},
            vdr_proxy::VdrProxySubmitter,
        },
        response_cacher::in_memory::{InMemoryResponseCacher, InMemoryResponseCacherConfig},
    },
    wallet::{
        base_wallet::BaseWallet,
        indy::{
            did_mocks::DidMocks,
            wallet::{create_and_open_wallet, create_and_store_my_did},
            WalletConfig,
        },
    },
    PoolConfig, ResponseParser, VdrProxyClient, WalletHandle,
};
use chrono::{DateTime, Duration, Utc};

use crate::{
    errors::error::VcxResult,
    global::settings,
    utils::{constants::POOL1_TXN, file::write_file, test_logger::LibvcxDefaultLogger},
};

const DEFAULT_AML_LABEL: &str = "eula";

lazy_static! {
    static ref TEST_LOGGING_INIT: Once = Once::new();
}

pub fn init_test_logging() {
    TEST_LOGGING_INIT.call_once(|| {
        LibvcxDefaultLogger::init_testing_logger();
    })
}

pub type DefaultIndyLedgerRead = IndyVdrLedgerRead<IndyVdrSubmitter, InMemoryResponseCacher>;
pub type DefaultIndyLedgerWrite = IndyVdrLedgerWrite<IndyVdrSubmitter>;

pub struct VcxPoolConfig {
    pub genesis_file_path: String,
    pub indy_vdr_config: Option<PoolConfig>,
    pub response_cache_config: Option<InMemoryResponseCacherConfig>,
}

pub fn build_ledger_components(
    pool_config: VcxPoolConfig,
) -> VcxResult<(DefaultIndyLedgerRead, DefaultIndyLedgerWrite)> {
    let indy_vdr_config = match pool_config.indy_vdr_config {
        None => PoolConfig::default(),
        Some(cfg) => cfg,
    };
    let cache_config = match pool_config.response_cache_config {
        None => InMemoryResponseCacherConfig::builder()
            .ttl(std::time::Duration::from_secs(60))
            .capacity(1000)?
            .build(),
        Some(cfg) => cfg,
    };

    let ledger_pool =
        IndyVdrLedgerPool::new(pool_config.genesis_file_path, indy_vdr_config, vec![])?;

    let request_submitter = IndyVdrSubmitter::new(ledger_pool);

    let ledger_read = indyvdr_build_ledger_read(request_submitter.clone(), cache_config)?;
    let ledger_write = indyvdr_build_ledger_write(request_submitter, None);

    Ok((ledger_read, ledger_write))
}

pub fn indyvdr_build_ledger_read(
    request_submitter: IndyVdrSubmitter,
    cache_config: InMemoryResponseCacherConfig,
) -> VcxResult<IndyVdrLedgerRead<IndyVdrSubmitter, InMemoryResponseCacher>> {
    let response_parser = ResponseParser;
    let response_cacher = InMemoryResponseCacher::new(cache_config);

    let config_read = IndyVdrLedgerReadConfig {
        request_submitter,
        response_parser,
        response_cacher,
        protocol_version: ProtocolVersion::Node1_4,
    };
    Ok(IndyVdrLedgerRead::new(config_read))
}

pub fn indyvdr_build_ledger_write(
    request_submitter: IndyVdrSubmitter,
    taa_options: Option<TxnAuthrAgrmtOptions>,
) -> IndyVdrLedgerWrite<IndyVdrSubmitter> {
    let config_write = IndyVdrLedgerWriteConfig {
        request_submitter,
        taa_options,
        protocol_version: ProtocolVersion::Node1_4,
    };
    IndyVdrLedgerWrite::new(config_write)
}

pub struct SetupMocks;

pub const AGENCY_ENDPOINT: &str = "http://localhost:8080";
pub const AGENCY_DID: &str = "VsKV7grR1BUE29mG2Fm2kX";
pub const AGENCY_VERKEY: &str = "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR";

pub struct SetupProfile<LR, LW, A, W>
where
    LR: IndyLedgerRead + AnoncredsLedgerRead,
    LW: IndyLedgerWrite + AnoncredsLedgerWrite,
    A: BaseAnonCreds,
    W: BaseWallet,
{
    pub institution_did: String,
    pub ledger_read: LR,
    pub ledger_write: LW,
    pub anoncreds: A,
    pub wallet: W,
    pub genesis_file_path: String,
}

pub async fn prepare_taa_options(
    ledger_read: &impl IndyLedgerRead,
) -> VcxResult<Option<TxnAuthrAgrmtOptions>> {
    if let Some(taa_result) = ledger_read.get_txn_author_agreement().await? {
        let taa_result: GetTxnAuthorAgreementData = serde_json::from_str(&taa_result)?;
        Ok(Some(TxnAuthrAgrmtOptions {
            version: taa_result.version,
            text: taa_result.text,
            mechanism: DEFAULT_AML_LABEL.to_string(),
        }))
    } else {
        Ok(None)
    }
}

pub struct SetupPoolDirectory {
    pub genesis_file_path: String,
}

pub fn reset_global_state() {
    warn!("reset_global_state >>");
    AgencyMockDecrypted::clear_mocks();
    DidMocks::clear_mocks();
}

impl SetupMocks {
    pub fn init() -> SetupMocks {
        init_test_logging();
        enable_agency_mocks();
        SetupMocks
    }
}

impl Drop for SetupMocks {
    fn drop(&mut self) {
        reset_global_state();
    }
}

pub async fn dev_setup_wallet_indy(key_seed: &str) -> (String, WalletHandle) {
    info!("dev_setup_wallet_indy >>");
    let config_wallet = WalletConfig {
        wallet_name: format!("wallet_{}", uuid::Uuid::new_v4()),
        wallet_key: settings::DEFAULT_WALLET_KEY.into(),
        wallet_key_derivation: settings::WALLET_KDF_RAW.into(),
        wallet_type: None,
        storage_config: None,
        storage_credentials: None,
        rekey: None,
        rekey_derivation_method: None,
    };
    let wallet_handle = create_and_open_wallet(&config_wallet).await.unwrap();
    // todo: can we just extract this away? not always we end up using it (alice test agent)
    let (did, _vk) = create_and_store_my_did(wallet_handle, Some(key_seed), None)
        .await
        .unwrap();

    (did, wallet_handle)
}

#[cfg(all(feature = "credx", feature = "vdrtools_wallet"))]
pub fn dev_build_profile_modular(
    genesis_file_path: String,
) -> (
    DefaultIndyLedgerRead,
    DefaultIndyLedgerWrite,
    IndyCredxAnonCreds,
) {
    info!("dev_build_profile_modular >>");
    let vcx_pool_config = VcxPoolConfig {
        genesis_file_path,
        indy_vdr_config: None,
        response_cache_config: None,
    };

    let anoncreds = IndyCredxAnonCreds;
    let (ledger_read, ledger_write) = build_ledger_components(vcx_pool_config).unwrap();

    (ledger_read, ledger_write, anoncreds)
}

#[cfg(feature = "vdr_proxy_ledger")]
pub async fn dev_build_profile_vdr_proxy_ledger() -> (
    IndyVdrLedgerRead<VdrProxySubmitter, InMemoryResponseCacher>,
    IndyVdrLedgerWrite<VdrProxySubmitter>,
    IndyCredxAnonCreds,
) {
    info!("dev_build_profile_vdr_proxy_ledger >>");

    let client_url =
        env::var("VDR_PROXY_CLIENT_URL").unwrap_or_else(|_| "http://127.0.0.1:3030".to_string());
    let client = VdrProxyClient::new(&client_url).unwrap();

    let anoncreds = IndyCredxAnonCreds;
    let request_submitter = VdrProxySubmitter::new(Arc::new(client));
    let response_parser = ResponseParser;
    let cacher_config = InMemoryResponseCacherConfig::builder()
        .ttl(std::time::Duration::from_secs(60))
        .capacity(1000)
        .unwrap()
        .build();
    let response_cacher = InMemoryResponseCacher::new(cacher_config);

    let config_read = IndyVdrLedgerReadConfig {
        request_submitter: request_submitter.clone(),
        response_parser,
        response_cacher,
        protocol_version: ProtocolVersion::Node1_4,
    };
    let ledger_read = IndyVdrLedgerRead::new(config_read);

    let config_write = IndyVdrLedgerWriteConfig {
        request_submitter,
        taa_options: prepare_taa_options(&ledger_read).await.unwrap(),
        protocol_version: ProtocolVersion::Node1_4,
    };
    let ledger_write = IndyVdrLedgerWrite::new(config_write);

    (ledger_read, ledger_write, anoncreds)
}

#[allow(unreachable_code)]
#[allow(unused_variables)]
pub async fn dev_build_featured_profile(
    genesis_file_path: String,
) -> (
    impl IndyLedgerRead + AnoncredsLedgerRead,
    impl IndyLedgerWrite + AnoncredsLedgerWrite,
    impl BaseAnonCreds,
) {
    #[cfg(feature = "vdr_proxy_ledger")]
    return {
        info!("SetupProfile >> using vdr proxy profile");
        dev_build_profile_vdr_proxy_ledger().await
    };

    #[cfg(all(
        feature = "credx",
        feature = "vdrtools_wallet",
        not(feature = "vdr_proxy_ledger")
    ))]
    return {
        info!("SetupProfile >> using modular profile");
        dev_build_profile_modular(genesis_file_path)
    };

    #[cfg(not(any(
        all(feature = "credx", feature = "vdrtools_wallet"),
        feature = "vdr_proxy_ledger"
    )))]
    (MockLedger, MockLedger, MockAnoncreds)
}

#[macro_export]
macro_rules! run_setup {
    ($func:expr) => {{
        use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;

        $crate::utils::devsetup::init_test_logging();

        let genesis_file_path = aries_vcx_core::ledger::indy::pool::test_utils::get_temp_file_path(
            $crate::utils::constants::POOL1_TXN,
        )
        .to_str()
        .unwrap()
        .to_string();
        aries_vcx_core::ledger::indy::pool::test_utils::create_testpool_genesis_txn_file(
            &genesis_file_path,
        );

        let (public_did, wallet_handle) =
            $crate::utils::devsetup::dev_setup_wallet_indy($crate::utils::constants::TRUSTEE_SEED)
                .await;
        let wallet = aries_vcx_core::wallet::indy::IndySdkWallet::new(wallet_handle);
        let (ledger_read, ledger_write, anoncreds) =
            $crate::utils::devsetup::dev_build_featured_profile(genesis_file_path.clone()).await;
        anoncreds
            .prover_create_link_secret(
                &wallet,
                aries_vcx_core::global::settings::DEFAULT_LINK_SECRET_ALIAS,
            )
            .await
            .unwrap();

        $crate::utils::devsetup::SetupProfile::new(
            ledger_read,
            ledger_write,
            anoncreds,
            wallet,
            public_did.to_string(),
            genesis_file_path,
        )
        .await
        .run($func)
    }};
}

impl<LR, LW, A, W> SetupProfile<LR, LW, A, W>
where
    LR: IndyLedgerRead + AnoncredsLedgerRead,
    LW: IndyLedgerWrite + AnoncredsLedgerWrite,
    A: BaseAnonCreds,
    W: BaseWallet,
{
    pub async fn new(
        ledger_read: LR,
        ledger_write: LW,
        anoncreds: A,
        wallet: W,
        institution_did: String,
        genesis_file_path: String,
    ) -> Self {
        debug!("genesis_file_path: {}", genesis_file_path);
        SetupProfile {
            institution_did,
            genesis_file_path,
            ledger_read,
            ledger_write,
            anoncreds,
            wallet,
        }
    }

    pub async fn run<F>(self, f: impl FnOnce(Self) -> F)
    where
        F: Future<Output = ()>,
    {
        f(self).await;
        reset_global_state();
    }
}

impl SetupPoolDirectory {
    async fn init() -> SetupPoolDirectory {
        debug!("SetupPoolDirectory init >> going to setup agency environment");
        init_test_logging();

        let genesis_file_path = get_temp_file_path(POOL1_TXN).to_str().unwrap().to_string();
        create_testpool_genesis_txn_file(&genesis_file_path);

        debug!("SetupPoolDirectory init >> completed");
        SetupPoolDirectory { genesis_file_path }
    }

    pub async fn run<F>(f: impl FnOnce(Self) -> F)
    where
        F: Future<Output = ()>,
    {
        let init = Self::init().await;

        f(init).await;

        reset_global_state();
    }
}

pub struct TempFile {
    pub path: String,
}

impl TempFile {
    pub fn prepare_path(filename: &str) -> TempFile {
        let file_path = get_temp_file_path(filename).to_str().unwrap().to_string();
        TempFile { path: file_path }
    }

    pub fn create(filename: &str) -> TempFile {
        let file_path = get_temp_file_path(filename).to_str().unwrap().to_string();
        fs::File::create(&file_path).unwrap();
        TempFile { path: file_path }
    }

    pub fn create_with_data(filename: &str, data: &str) -> TempFile {
        let mut file = TempFile::create(filename);
        file.write(data);
        file
    }

    pub fn write(&mut self, data: &str) {
        write_file(&self.path, data).unwrap()
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        fs::remove_file(&self.path).unwrap_or(());
    }
}

pub fn was_in_past(datetime_rfc3339: &str, threshold: Duration) -> chrono::ParseResult<bool> {
    let now = Utc::now();
    let datetime: DateTime<Utc> = DateTime::parse_from_rfc3339(datetime_rfc3339)?.into();
    let diff = now - datetime;
    Ok(threshold > diff)
}

#[cfg(test)]
pub mod unit_tests {
    use std::ops::Sub;

    use chrono::SecondsFormat;

    use super::*;

    #[test]
    fn test_is_past_timestamp() {
        let now = Utc::now();
        let past1ms_rfc3339 = now
            .sub(Duration::milliseconds(1))
            .to_rfc3339_opts(SecondsFormat::Millis, true);
        assert!(was_in_past(&past1ms_rfc3339, Duration::milliseconds(10)).unwrap());
    }
}
