#![allow(unused_imports)]

use std::{
    env,
    error::Error,
    fs::{self, DirBuilder, OpenOptions},
    future::Future,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Once},
};

use agency_client::testing::mocking::{enable_agency_mocks, AgencyMockDecrypted};
#[cfg(feature = "credx")]
use aries_vcx_core::anoncreds::credx_anoncreds::IndyCredxAnonCreds;
use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    global::settings::{DEFAULT_WALLET_KEY, WALLET_KDF_RAW},
    ledger::{
        base_ledger::{
            AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite,
            TxnAuthrAgrmtOptions,
        },
        indy::pool::test_utils::{create_testpool_genesis_txn_file, get_temp_file_path},
        indy_vdr_ledger::{
            build_ledger_components, indyvdr_build_ledger_read, indyvdr_build_ledger_write,
            DefaultIndyLedgerRead, DefaultIndyLedgerWrite, GetTxnAuthorAgreementData,
            IndyVdrLedgerRead, IndyVdrLedgerReadConfig, IndyVdrLedgerWrite,
            IndyVdrLedgerWriteConfig, ProtocolVersion, VcxPoolConfig,
        },
        request_submitter::vdr_ledger::{IndyVdrLedgerPool, IndyVdrSubmitter},
        response_cacher::in_memory::{InMemoryResponseCacher, InMemoryResponseCacherConfig},
    },
    wallet::{base_wallet::BaseWallet, mock_wallet::MockWallet},
    PoolConfig, ResponseParser,
};
#[cfg(feature = "vdr_proxy_ledger")]
use aries_vcx_core::{ledger::request_submitter::vdr_proxy::VdrProxySubmitter, VdrProxyClient};
#[cfg(feature = "vdrtools_wallet")]
use aries_vcx_core::{
    wallet::indy::{
        wallet::{create_and_open_wallet, create_and_store_my_did},
        WalletConfig,
    },
    WalletHandle,
};
use chrono::{DateTime, Duration, Utc};
use lazy_static::lazy_static;
use libvcx_logger::init_test_logging;
use log::{debug, info, warn};

use crate::{
    constants::{INSTITUTION_DID, POOL1_TXN, TRUSTEE_SEED},
    mockdata::{mock_anoncreds::MockAnoncreds, mock_ledger::MockLedger},
};

const DEFAULT_AML_LABEL: &str = "eula";

pub fn write_file<P: AsRef<Path>>(file: P, content: &str) -> VcxCoreResult<()>
where
    P: std::convert::AsRef<std::ffi::OsStr>,
{
    let path = PathBuf::from(&file);

    if let Some(parent_path) = path.parent() {
        DirBuilder::new()
            .recursive(true)
            .create(parent_path)
            .map_err(|err| {
                AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::UnknownError,
                    format!("Can't create the file: {}", err),
                )
            })?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .map_err(|err| {
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::UnknownError,
                format!("Can't open the file: {}", err),
            )
        })?;

    file.write_all(content.as_bytes()).map_err(|err| {
        AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnknownError,
            format!("Can't write content: \"{}\" to the file: {}", content, err),
        )
    })?;

    file.flush().map_err(|err| {
        AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnknownError,
            format!("Can't write content: \"{}\" to the file: {}", content, err),
        )
    })?;

    file.sync_data().map_err(|err| {
        AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::UnknownError,
            format!("Can't write content: \"{}\" to the file: {}", content, err),
        )
    })
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
) -> VcxCoreResult<Option<TxnAuthrAgrmtOptions>> {
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

#[cfg(feature = "vdrtools_wallet")]
pub async fn dev_setup_wallet_indy(key_seed: &str) -> (String, WalletHandle) {
    info!("dev_setup_wallet_indy >>");
    let config_wallet = WalletConfig {
        wallet_name: format!("wallet_{}", uuid::Uuid::new_v4()),
        wallet_key: DEFAULT_WALLET_KEY.into(),
        wallet_key_derivation: WALLET_KDF_RAW.into(),
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
pub fn dev_build_profile_vdr_ledger(
    genesis_file_path: String,
) -> (DefaultIndyLedgerRead, DefaultIndyLedgerWrite) {
    info!("dev_build_profile_vdr_ledger >>");
    let vcx_pool_config = VcxPoolConfig {
        genesis_file_path,
        indy_vdr_config: None,
        response_cache_config: None,
    };

    let (ledger_read, ledger_write) = build_ledger_components(vcx_pool_config).unwrap();

    (ledger_read, ledger_write)
}

#[cfg(feature = "vdr_proxy_ledger")]
pub async fn dev_build_profile_vdr_proxy_ledger() -> (
    IndyVdrLedgerRead<VdrProxySubmitter, InMemoryResponseCacher>,
    IndyVdrLedgerWrite<VdrProxySubmitter>,
) {
    info!("dev_build_profile_vdr_proxy_ledger >>");

    let client_url =
        env::var("VDR_PROXY_CLIENT_URL").unwrap_or_else(|_| "http://127.0.0.1:3030".to_string());
    let client = VdrProxyClient::new(&client_url).unwrap();

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

    (ledger_read, ledger_write)
}

#[allow(unreachable_code)]
#[allow(unused_variables)]
pub async fn dev_build_featured_indy_ledger(
    genesis_file_path: String,
) -> (
    impl IndyLedgerRead + AnoncredsLedgerRead,
    impl IndyLedgerWrite + AnoncredsLedgerWrite,
) {
    #[cfg(feature = "vdr_proxy_ledger")]
    return {
        info!("SetupProfile >> using vdr proxy ldeger");
        dev_build_profile_vdr_proxy_ledger().await
    };

    #[cfg(not(feature = "vdr_proxy_ledger"))]
    return {
        info!("SetupProfile >> using vdr ledger");
        dev_build_profile_vdr_ledger(genesis_file_path)
    };
}

#[cfg(feature = "vdrtools_wallet")]
pub async fn dev_build_indy_wallet(key_seed: &str) -> (String, impl BaseWallet) {
    use aries_vcx_core::wallet::indy::IndySdkWallet;

    let (public_did, wallet_handle) = dev_setup_wallet_indy(key_seed).await;
    (public_did, IndySdkWallet::new(wallet_handle))
}

#[allow(unreachable_code)]
#[allow(unused_variables)]
pub async fn dev_build_featured_anoncreds() -> impl BaseAnonCreds {
    #[cfg(feature = "credx")]
    return IndyCredxAnonCreds;

    #[cfg(not(feature = "credx"))]
    return MockAnoncreds;
}

#[allow(unreachable_code)]
#[allow(unused_variables)]
pub async fn dev_build_featured_wallet(key_seed: &str) -> (String, impl BaseWallet) {
    #[cfg(feature = "vdrtools_wallet")]
    return {
        info!("SetupProfile >> using indy wallet");
        dev_build_indy_wallet(key_seed).await
    };

    #[cfg(not(feature = "vdrtools_wallet"))]
    return (INSTITUTION_DID.to_owned(), MockWallet);
}

pub async fn build_setup_profile() -> SetupProfile<
    impl IndyLedgerRead + AnoncredsLedgerRead,
    impl IndyLedgerWrite + AnoncredsLedgerWrite,
    impl BaseAnonCreds,
    impl BaseWallet,
> {
    use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;

    init_test_logging();

    let genesis_file_path = get_temp_file_path(POOL1_TXN).to_str().unwrap().to_string();
    create_testpool_genesis_txn_file(&genesis_file_path);

    let (institution_did, wallet) = dev_build_featured_wallet(TRUSTEE_SEED).await;
    let (ledger_read, ledger_write) =
        dev_build_featured_indy_ledger(genesis_file_path.clone()).await;
    let anoncreds = dev_build_featured_anoncreds().await;

    anoncreds
        .prover_create_link_secret(
            &wallet,
            aries_vcx_core::global::settings::DEFAULT_LINK_SECRET_ALIAS,
        )
        .await
        .unwrap();

    debug!("genesis_file_path: {}", genesis_file_path);

    SetupProfile {
        ledger_read,
        ledger_write,
        anoncreds,
        wallet,
        institution_did,
        genesis_file_path,
    }
}

impl SetupPoolDirectory {
    pub async fn init() -> SetupPoolDirectory {
        debug!("SetupPoolDirectory init >> going to setup agency environment");
        init_test_logging();

        let genesis_file_path = get_temp_file_path(POOL1_TXN).to_str().unwrap().to_string();
        create_testpool_genesis_txn_file(&genesis_file_path);

        debug!("SetupPoolDirectory init >> completed");
        SetupPoolDirectory { genesis_file_path }
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
