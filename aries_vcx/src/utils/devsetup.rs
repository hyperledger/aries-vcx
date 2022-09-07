use chrono::{DateTime, Duration, Utc};
use std::fs;
use std::sync::Once;

use indy_sys::{WalletHandle, PoolHandle};

use agency_client::agency_client::AgencyClient;
use agency_client::configuration::AgentProvisionConfig;
use agency_client::testing::mocking::{disable_agency_mocks, enable_agency_mocks, AgencyMockDecrypted};

use crate::global::pool::reset_main_pool_handle;
use crate::global::settings;
use crate::global::settings::init_issuer_config;
use crate::global::settings::{disable_indy_mocks, enable_indy_mocks, set_test_configs};
use crate::libindy::utils::mocks::did_mocks::DidMocks;
use crate::libindy::utils::mocks::pool_mocks::PoolMocks;
use crate::libindy::utils::pool::test_utils::{create_test_ledger_config, delete_test_pool, open_test_pool};
use crate::libindy::utils::pool::PoolConfig;
use crate::libindy::utils::wallet::wallet_configure_issuer;
use crate::libindy::utils::wallet::{
    close_wallet, create_and_open_wallet, create_indy_wallet, create_wallet_with_master_secret, delete_wallet,
    WalletConfig,
};
use crate::libindy::wallet::open_wallet;
use crate::utils;
use crate::utils::file::write_file;
use crate::utils::get_temp_dir_path;
use crate::utils::provision::provision_cloud_agent;
use crate::utils::test_logger::LibvcxDefaultLogger;

pub struct SetupEmpty;

pub struct SetupDefaults;

pub struct SetupMocks {
    pub institution_did: String
}

pub struct SetupIndyMocks;

pub struct TestSetupCreateWallet {
    pub wallet_config: WalletConfig,
    skip_cleanup: bool,
}

pub struct SetupPoolConfig {
    skip_cleanup: bool,
    pub pool_config: PoolConfig,
}

pub struct SetupLibraryWallet {
    pub wallet_config: WalletConfig,
    pub wallet_handle: WalletHandle,
}

pub struct SetupWalletPoolAgency {
    pub agency_client: AgencyClient,
    pub institution_did: String,
    pub wallet_handle: WalletHandle,
}

pub struct SetupWalletPool {
    pub institution_did: String,
    pub wallet_handle: WalletHandle,
    pub pool_handle: PoolHandle
}

pub struct SetupInstitutionWallet {
    pub institution_did: String,
    pub wallet_handle: WalletHandle,
}

pub struct SetupLibraryAgencyV2;

fn reset_global_state() {
    warn!("reset_global_state >>");
    AgencyMockDecrypted::clear_mocks();
    PoolMocks::clear_mocks();
    DidMocks::clear_mocks();
    reset_main_pool_handle();
    disable_indy_mocks().unwrap();
    settings::reset_config_values();
}

impl SetupEmpty {
    pub fn init() -> SetupEmpty {
        init_test_logging();
        SetupEmpty {}
    }
}

impl Drop for SetupEmpty {
    fn drop(&mut self) {
        reset_global_state();
    }
}

impl SetupDefaults {
    pub fn init() -> SetupDefaults {
        init_test_logging();
        set_test_configs();
        SetupDefaults {}
    }
}

impl Drop for SetupDefaults {
    fn drop(&mut self) {
        reset_global_state();
    }
}

impl SetupMocks {
    pub fn init() -> SetupMocks {
        init_test_logging();
        let institution_did = set_test_configs();
        enable_agency_mocks();
        enable_indy_mocks().unwrap();
        SetupMocks { institution_did }
    }
}

impl Drop for SetupMocks {
    fn drop(&mut self) {
        reset_global_state();
    }
}

impl SetupLibraryWallet {
    pub async fn init() -> SetupLibraryWallet {
        init_test_logging();
        debug!("SetupLibraryWallet::init >>");
        set_test_configs();
        let wallet_name: String = format!("Test_SetupLibraryWallet_{}", uuid::Uuid::new_v4().to_string());
        let wallet_key: String = settings::DEFAULT_WALLET_KEY.into();
        let wallet_kdf: String = settings::WALLET_KDF_RAW.into();
        let wallet_config = WalletConfig {
            wallet_name: wallet_name.clone(),
            wallet_key: wallet_key.clone(),
            wallet_key_derivation: wallet_kdf.to_string(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };

        let wallet_handle = create_and_open_wallet(&wallet_config).await.unwrap();
        SetupLibraryWallet {
            wallet_config,
            wallet_handle,
        }
    }
}

impl Drop for SetupLibraryWallet {
    fn drop(&mut self) {
        let _res = futures::executor::block_on(close_wallet(self.wallet_handle)).unwrap();
        futures::executor::block_on(delete_wallet(&self.wallet_config)).unwrap();
        reset_global_state();
    }
}

impl TestSetupCreateWallet {
    pub async fn init() -> TestSetupCreateWallet {
        init_test_logging();
        set_test_configs();
        let wallet_name: String = format!("Test_SetupWallet_{}", uuid::Uuid::new_v4().to_string());
        disable_agency_mocks();
        let wallet_config = WalletConfig {
            wallet_name: wallet_name.clone(),
            wallet_key: settings::DEFAULT_WALLET_KEY.into(),
            wallet_key_derivation: settings::WALLET_KDF_RAW.into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };
        create_indy_wallet(&wallet_config).await.unwrap();

        TestSetupCreateWallet {
            wallet_config,
            skip_cleanup: false,
        }
    }

    pub fn skip_cleanup(mut self) -> TestSetupCreateWallet {
        self.skip_cleanup = true;
        self
    }
}

impl Drop for TestSetupCreateWallet {
    fn drop(&mut self) {
        if self.skip_cleanup == false {
            futures::executor::block_on(delete_wallet(&self.wallet_config))
                .unwrap_or_else(|_e| error!("Failed to delete wallet while dropping SetupWallet test config."));
        }
        reset_global_state();
    }
}

impl SetupPoolConfig {
    pub async fn init() -> SetupPoolConfig {
        init_test_logging();

        create_test_ledger_config().await;
        let genesis_path = utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH)
            .to_str()
            .unwrap()
            .to_string();
        let pool_config = PoolConfig {
            genesis_path,
            pool_name: None,
            pool_config: None,
        };

        SetupPoolConfig {
            skip_cleanup: false,
            pool_config,
        }
    }

    pub fn skip_cleanup(mut self) -> SetupPoolConfig {
        self.skip_cleanup = true;
        self
    }
}

impl Drop for SetupPoolConfig {
    fn drop(&mut self) {
        if self.skip_cleanup == false {
            futures::executor::block_on(delete_test_pool());
            reset_main_pool_handle();
        }
        reset_global_state();
    }
}

impl SetupIndyMocks {
    pub fn init() -> SetupIndyMocks {
        init_test_logging();
        enable_indy_mocks().unwrap();
        enable_agency_mocks();
        SetupIndyMocks {}
    }
}

impl Drop for SetupIndyMocks {
    fn drop(&mut self) {
        reset_global_state();
    }
}

impl SetupWalletPoolAgency {
    pub async fn init() -> SetupWalletPoolAgency {
        init_test_logging();
        set_test_configs();
        let (institution_did, wallet_handle, agency_client) = setup_issuer_wallet_and_agency_client().await;
        settings::set_config_value(
            settings::CONFIG_GENESIS_PATH,
            utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH)
                .to_str()
                .unwrap(),
        )
        .unwrap();
        open_test_pool().await;
        SetupWalletPoolAgency {
            agency_client,
            institution_did,
            wallet_handle,
        }
    }
}

impl Drop for SetupWalletPoolAgency {
    fn drop(&mut self) {
        futures::executor::block_on(delete_test_pool());
        reset_global_state();
    }
}

impl SetupWalletPool {
    pub async fn init() -> SetupWalletPool {
        init_test_logging();
        set_test_configs();
        let (institution_did, wallet_handle) = setup_issuer_wallet().await;
        settings::set_config_value(
            settings::CONFIG_GENESIS_PATH,
            utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH)
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let pool_handle = open_test_pool().await;
        SetupWalletPool {
            institution_did,
            wallet_handle,
            pool_handle
        }
    }
}

impl Drop for SetupWalletPool {
    fn drop(&mut self) {
        futures::executor::block_on(delete_test_pool());
        reset_global_state();
    }
}

impl SetupInstitutionWallet {
    pub async fn init() -> SetupInstitutionWallet {
        init_test_logging();
        set_test_configs();
        let (institution_did, wallet_handle) = setup_issuer_wallet().await;
        SetupInstitutionWallet {
            institution_did,
            wallet_handle,
        }
    }
}

impl Drop for SetupInstitutionWallet {
    fn drop(&mut self) {
        reset_global_state();
    }
}

impl SetupLibraryAgencyV2 {
    pub async fn init() -> SetupLibraryAgencyV2 {
        debug!("SetupLibraryAgencyV2 init >> going to setup agency environment");
        init_test_logging();

        settings::set_config_value(
            settings::CONFIG_GENESIS_PATH,
            utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH)
                .to_str()
                .unwrap(),
        )
        .unwrap();
        open_test_pool().await;
        debug!("SetupLibraryAgencyV2 init >> completed");
        SetupLibraryAgencyV2
    }
}

impl Drop for SetupLibraryAgencyV2 {
    fn drop(&mut self) {
        futures::executor::block_on(delete_test_pool());
        reset_global_state();
    }
}

#[macro_export]
macro_rules! assert_match {
    ($pattern:pat, $var:expr) => {
        assert!(match $var {
            $pattern => true,
            _ => false,
        })
    };
}

pub const AGENCY_ENDPOINT: &'static str = "http://localhost:8080";
pub const AGENCY_DID: &'static str = "VsKV7grR1BUE29mG2Fm2kX";
pub const AGENCY_VERKEY: &'static str = "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR";

lazy_static! {
    static ref TEST_LOGGING_INIT: Once = Once::new();
}

pub fn init_test_logging() {
    TEST_LOGGING_INIT.call_once(|| {
        LibvcxDefaultLogger::init_testing_logger();
    })
}

pub fn create_new_seed() -> String {
    let x = rand::random::<u32>();
    format!("{:032}", x)
}

pub async fn setup_issuer_wallet_and_agency_client() -> (String, WalletHandle, AgencyClient) {
    let enterprise_seed = "000000000000000000000000Trustee1";
    let config_wallet = WalletConfig {
        wallet_name: format!("wallet_{}", uuid::Uuid::new_v4().to_string()),
        wallet_key: settings::DEFAULT_WALLET_KEY.into(),
        wallet_key_derivation: settings::WALLET_KDF_RAW.into(),
        wallet_type: None,
        storage_config: None,
        storage_credentials: None,
        rekey: None,
        rekey_derivation_method: None,
    };
    let config_provision_agent = AgentProvisionConfig {
        agency_did: AGENCY_DID.to_string(),
        agency_verkey: AGENCY_VERKEY.to_string(),
        agency_endpoint: AGENCY_ENDPOINT.to_string(),
        agent_seed: None,
    };
    create_wallet_with_master_secret(&config_wallet).await.unwrap();
    let wallet_handle = open_wallet(&config_wallet).await.unwrap();
    let config_issuer = wallet_configure_issuer(wallet_handle, enterprise_seed).await.unwrap();
    init_issuer_config(&config_issuer).unwrap();
    let mut agency_client = AgencyClient::new();
    provision_cloud_agent(&mut agency_client, wallet_handle, &config_provision_agent)
        .await
        .unwrap();

    (config_issuer.institution_did, wallet_handle, agency_client)
}

pub async fn setup_issuer_wallet() -> (String, WalletHandle) {
    let enterprise_seed = "000000000000000000000000Trustee1";
    let config_wallet = WalletConfig {
        wallet_name: format!("wallet_{}", uuid::Uuid::new_v4().to_string()),
        wallet_key: settings::DEFAULT_WALLET_KEY.into(),
        wallet_key_derivation: settings::WALLET_KDF_RAW.into(),
        wallet_type: None,
        storage_config: None,
        storage_credentials: None,
        rekey: None,
        rekey_derivation_method: None,
    };
    create_wallet_with_master_secret(&config_wallet).await.unwrap();
    let wallet_handle = open_wallet(&config_wallet).await.unwrap();
    let config_issuer = wallet_configure_issuer(wallet_handle, enterprise_seed).await.unwrap();
    init_issuer_config(&config_issuer).unwrap();
    (config_issuer.institution_did, wallet_handle)
}

pub struct TempFile {
    pub path: String,
}

impl TempFile {
    pub fn prepare_path(filename: &str) -> TempFile {
        let file_path = get_temp_dir_path(filename).to_str().unwrap().to_string();
        TempFile { path: file_path }
    }

    pub fn create(filename: &str) -> TempFile {
        let file_path = get_temp_dir_path(filename).to_str().unwrap().to_string();
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
        fs::remove_file(&self.path).unwrap()
    }
}

#[cfg(feature = "test_utils")]
pub fn was_in_past(datetime_rfc3339: &str, threshold: Duration) -> chrono::ParseResult<bool> {
    let now = Utc::now();
    let datetime: DateTime<Utc> = DateTime::parse_from_rfc3339(datetime_rfc3339)?.into();
    let diff = now - datetime;
    Ok(threshold > diff)
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;
    use chrono::SecondsFormat;
    use std::ops::Sub;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_is_past_timestamp() {
        let now = Utc::now();
        let past1ms_rfc3339 = now
            .sub(Duration::milliseconds(1))
            .to_rfc3339_opts(SecondsFormat::Millis, true);
        assert!(was_in_past(&past1ms_rfc3339, Duration::milliseconds(10)).unwrap())
    }
}
