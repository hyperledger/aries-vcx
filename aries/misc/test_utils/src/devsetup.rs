use std::{
    fs::{self, DirBuilder, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use aries_vcx_anoncreds::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_ledger::{
    errors::error::VcxLedgerResult,
    ledger::{
        base_ledger::{
            AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite,
            TxnAuthrAgrmtOptions,
        },
        indy::pool::test_utils::{create_testpool_genesis_txn_file, get_temp_file_path},
        indy_vdr_ledger::{
            build_ledger_components, DefaultIndyLedgerRead, DefaultIndyLedgerWrite,
            GetTxnAuthorAgreementData, VcxPoolConfig,
        },
    },
};
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use chrono::{DateTime, Duration, Utc};
use did_parser_nom::Did;
use log::{debug, info};

use crate::{
    constants::{POOL1_TXN, TRUSTEE_SEED},
    errors::error::{TestUtilsError, TestUtilsResult},
    settings,
};

#[cfg(feature = "vdr_proxy_ledger")]
pub mod vdr_proxy_ledger;

#[cfg(feature = "vdr_proxy_ledger")]
use crate::devsetup::vdr_proxy_ledger::dev_build_profile_vdr_proxy_ledger;
use crate::logger::init_logger;

#[cfg(feature = "askar_wallet")]
pub mod askar_wallet;

const DEFAULT_AML_LABEL: &str = "eula";

pub fn write_file<P: AsRef<Path> + AsRef<std::ffi::OsStr>>(
    file: P,
    content: &str,
) -> TestUtilsResult<()> {
    let path = PathBuf::from(&file);

    if let Some(parent_path) = path.parent() {
        DirBuilder::new()
            .recursive(true)
            .create(parent_path)
            .map_err(|err| {
                TestUtilsError::UnknownError(format!("Can't create the file: {}", err))
            })?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .map_err(|err| TestUtilsError::UnknownError(format!("Can't open the file: {}", err)))?;

    file.write_all(content.as_bytes()).map_err(|err| {
        TestUtilsError::UnknownError(format!(
            "Can't write content: \"{}\" to the file: {}",
            content, err
        ))
    })?;

    file.flush().map_err(|err| {
        TestUtilsError::UnknownError(format!(
            "Can't write content: \"{}\" to the file: {}",
            content, err
        ))
    })?;

    file.sync_data().map_err(|err| {
        TestUtilsError::UnknownError(format!(
            "Can't write content: \"{}\" to the file: {}",
            content, err
        ))
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
    pub institution_did: Did,
    pub ledger_read: LR,
    pub ledger_write: LW,
    pub anoncreds: A,
    pub wallet: W,
    pub genesis_file_path: String,
}

pub async fn prepare_taa_options(
    ledger_read: &impl IndyLedgerRead,
) -> VcxLedgerResult<Option<TxnAuthrAgrmtOptions>> {
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

impl SetupMocks {
    pub fn init() -> SetupMocks {
        init_logger();
        SetupMocks
    }
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

#[allow(clippy::needless_return)]
pub async fn dev_build_featured_anoncreds() -> impl BaseAnonCreds {
    #[cfg(feature = "anoncreds")]
    {
        use aries_vcx_anoncreds::anoncreds::anoncreds::Anoncreds;
        return Anoncreds;
    }

    #[cfg(not(feature = "anoncreds"))]
    {
        use crate::mockdata::mock_anoncreds::MockAnoncreds;
        return MockAnoncreds;
    };
}

#[allow(unused_variables)]
pub async fn dev_build_featured_wallet(key_seed: &str) -> (String, impl BaseWallet) {
    #[cfg(feature = "askar_wallet")]
    return {
        info!("SetupProfile >> using indy wallet");

        use crate::devsetup::askar_wallet::dev_setup_wallet_askar;
        dev_setup_wallet_askar(key_seed).await
    };

    #[cfg(not(feature = "askar_wallet"))]
    {
        use crate::{constants::INSTITUTION_DID, mock_wallet::MockWallet};

        (INSTITUTION_DID.to_owned(), MockWallet)
    }
}

pub async fn build_setup_profile() -> SetupProfile<
    impl IndyLedgerRead + AnoncredsLedgerRead,
    impl IndyLedgerWrite + AnoncredsLedgerWrite,
    impl BaseAnonCreds,
    impl BaseWallet,
> {
    init_logger();

    let genesis_file_path = get_temp_file_path(POOL1_TXN).to_str().unwrap().to_string();
    create_testpool_genesis_txn_file(&genesis_file_path);

    let (institution_did, wallet) = dev_build_featured_wallet(TRUSTEE_SEED).await;
    let (ledger_read, ledger_write) =
        dev_build_featured_indy_ledger(genesis_file_path.clone()).await;
    let anoncreds = dev_build_featured_anoncreds().await;

    anoncreds
        .prover_create_link_secret(&wallet, &settings::DEFAULT_LINK_SECRET_ALIAS.to_string())
        .await
        .unwrap();

    debug!("genesis_file_path: {}", genesis_file_path);

    SetupProfile {
        ledger_read,
        ledger_write,
        anoncreds,
        wallet,
        institution_did: Did::parse(institution_did).unwrap(),
        genesis_file_path,
    }
}

impl SetupPoolDirectory {
    pub async fn init() -> SetupPoolDirectory {
        debug!("SetupPoolDirectory init >> going to setup agency environment");
        init_logger();

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
