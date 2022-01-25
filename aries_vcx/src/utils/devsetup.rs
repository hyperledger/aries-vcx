use std::fs;
use std::sync::Once;

use crate::{libindy, settings, utils};
use crate::agency_client::mocking::AgencyMockDecrypted;
use crate::init::{init_issuer_config, open_as_main_wallet};
use crate::init::PoolConfig;
use crate::libindy::utils::pool::reset_pool_handle;
use crate::libindy::utils::pool::test_utils::{create_test_ledger_config, delete_test_pool, open_test_pool};
use crate::libindy::utils::wallet::{close_main_wallet, create_and_open_as_main_wallet, create_indy_wallet, delete_wallet, reset_wallet_handle, WalletConfig};
use crate::libindy::utils::wallet::{configure_issuer_wallet, create_wallet};
use crate::settings::set_testing_defaults;
use crate::utils::constants;
use crate::utils::file::write_file;
use crate::utils::get_temp_dir_path;
use crate::utils::provision::{AgentProvisionConfig, provision_cloud_agent};
use crate::utils::test_logger::LibvcxDefaultLogger;

pub struct SetupEmpty; // clears settings, setups up logging

pub struct SetupDefaults; // set default settings

pub struct SetupMocks; // set default settings and enable test mode

pub struct SetupIndyMocks; // set default settings and enable indy mode

pub struct SetupWallet {
    pub wallet_config: WalletConfig,
    skip_cleanup: bool,
} // creates wallet with random name, configures wallet settings

pub struct SetupPoolConfig {
    skip_cleanup: bool,
    pub pool_config: PoolConfig,
}

pub struct SetupLibraryWallet {
    pub wallet_config: WalletConfig,
} // set default settings and init indy wallet

pub struct SetupWithWalletAndAgency {
    pub institution_did: String,
}  // set default settings, init indy wallet, init pool

pub struct SetupAgencyMock {
    pub wallet_config: WalletConfig,
} // set default settings and enable mock agency mode

pub struct SetupLibraryAgencyV2; // init indy wallet, init pool, provision 2 agents. use protocol type 2.0

fn setup() {
    init_test_logging();
    settings::clear_config();
    set_testing_defaults();
}

fn setup_empty() {
    settings::clear_config();
    init_test_logging();
}

fn tear_down() {
    settings::clear_config();
    reset_wallet_handle().unwrap();
    reset_pool_handle();
    settings::get_agency_client_mut().unwrap().disable_test_mode();
    AgencyMockDecrypted::clear_mocks();
}

impl SetupEmpty {
    pub fn init() {
        setup_empty();
    }
}

impl Drop for SetupEmpty {
    fn drop(&mut self) {
        tear_down()
    }
}

impl SetupDefaults {
    pub fn init() {
        debug!("SetupDefaults :: starting");
        setup();
        debug!("SetupDefaults :: finished");
    }
}

impl Drop for SetupDefaults {
    fn drop(&mut self) {
        tear_down()
    }
}

impl SetupMocks {
    fn _init() -> SetupMocks {
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
        settings::get_agency_client_mut().unwrap().enable_test_mode();
        SetupMocks
    }

    pub fn init() -> SetupMocks {
        setup();
        SetupMocks::_init()
    }

    pub fn init_without_threadpool() -> SetupMocks {
        setup();
        SetupMocks::_init()
    }
}

impl Drop for SetupMocks {
    fn drop(&mut self) {
        tear_down()
    }
}

impl SetupLibraryWallet {
    pub fn init() -> SetupLibraryWallet {
        setup();
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

        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
        settings::get_agency_client_mut().unwrap().disable_test_mode();
        create_and_open_as_main_wallet(&wallet_config).unwrap();
        SetupLibraryWallet { wallet_config }
    }
}

impl Drop for SetupLibraryWallet {
    fn drop(&mut self) {
        let _res = close_main_wallet().unwrap();
        delete_wallet(&self.wallet_config).unwrap();
        tear_down()
    }
}

impl SetupWallet {
    pub fn init() -> SetupWallet {
        init_test_logging();
        let wallet_name: String = format!("Test_SetupWallet_{}", uuid::Uuid::new_v4().to_string());
        settings::get_agency_client_mut().unwrap().disable_test_mode();
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
        create_indy_wallet(&wallet_config).unwrap();

        SetupWallet { wallet_config, skip_cleanup: false }
    }

    pub fn skip_cleanup(mut self) -> SetupWallet {
        self.skip_cleanup = true;
        self
    }
}

impl Drop for SetupWallet {
    fn drop(&mut self) {
        if self.skip_cleanup == false {
            let _res = close_main_wallet().unwrap_or_else(|_e| error!("Failed to close main wallet while dropping SetupWallet test config."));
            delete_wallet(&self.wallet_config).unwrap_or_else(|_e| error!("Failed to delete wallet while dropping SetupWallet test config."));
            reset_wallet_handle().unwrap_or_else(|_e| error!("Failed to reset wallet handle while dropping SetupWallet test config."));
        }
    }
}

impl SetupPoolConfig {
    pub fn init() -> SetupPoolConfig {
        create_test_ledger_config();
        let genesis_path = utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH).to_str().unwrap().to_string();
        let pool_config = PoolConfig {
            genesis_path,
            pool_name: None,
            pool_config: None,
        };

        SetupPoolConfig { skip_cleanup: false, pool_config }
    }

    pub fn skip_cleanup(mut self) -> SetupPoolConfig {
        self.skip_cleanup = true;
        self
    }
}

impl Drop for SetupPoolConfig {
    fn drop(&mut self) {
        if self.skip_cleanup == false {
            delete_test_pool();
            reset_pool_handle();
        }
    }
}

impl SetupIndyMocks {
    pub fn init() -> SetupIndyMocks {
        setup();
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
        settings::get_agency_client_mut().unwrap().enable_test_mode();
        SetupIndyMocks {}
    }
}

impl Drop for SetupIndyMocks {
    fn drop(&mut self) {
        tear_down()
    }
}

impl SetupWithWalletAndAgency {
    pub async fn init() -> SetupWithWalletAndAgency {
        setup();
        let institution_did = setup_indy_env().await;
        SetupWithWalletAndAgency {
            institution_did
        }
    }
}

impl Drop for SetupWithWalletAndAgency {
    fn drop(&mut self) {
        cleanup_indy_env();
        tear_down()
    }
}

impl SetupAgencyMock {
    pub fn init() -> SetupAgencyMock {
        setup();
        let wallet_name: String = format!("Test_SetupWalletAndPool_{}", uuid::Uuid::new_v4().to_string());
        settings::get_agency_client_mut().unwrap().enable_test_mode();

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
        create_and_open_as_main_wallet(&wallet_config).unwrap();

        SetupAgencyMock { wallet_config }
    }
}

impl Drop for SetupAgencyMock {
    fn drop(&mut self) {
        let _res = close_main_wallet().unwrap();
        delete_wallet(&self.wallet_config).unwrap();
        tear_down()
    }
}

impl SetupLibraryAgencyV2 {
    pub fn init() -> SetupLibraryAgencyV2 {
        setup();
        debug!("SetupLibraryAgencyV2 init >> going to setup agency environment");
        setup_agency_env();
        debug!("SetupLibraryAgencyV2 init >> completed");
        SetupLibraryAgencyV2
    }
}

impl Drop for SetupLibraryAgencyV2 {
    fn drop(&mut self) {
        cleanup_agency_env();
        tear_down()
    }
}

#[macro_export]
macro_rules! assert_match {
    ($pattern:pat, $var:expr) => (
        assert!(match $var {
            $pattern => true,
            _ => false
        })
    );
}


/* dummy */
pub const AGENCY_ENDPOINT: &'static str = "http://localhost:8080";
pub const AGENCY_DID: &'static str = "VsKV7grR1BUE29mG2Fm2kX";
pub const AGENCY_VERKEY: &'static str = "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR";

pub const C_AGENCY_ENDPOINT: &'static str = "http://localhost:8080";
pub const C_AGENCY_DID: &'static str = "VsKV7grR1BUE29mG2Fm2kX";
pub const C_AGENCY_VERKEY: &'static str = "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR";


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

pub fn configure_trustee_did() {
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
    libindy::utils::anoncreds::libindy_prover_create_master_secret(settings::DEFAULT_LINK_SECRET_ALIAS).unwrap();
    let (my_did, my_vk) = libindy::utils::signus::create_and_store_my_did(Some(constants::TRUSTEE_SEED), None).unwrap();
    settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &my_did);
    settings::set_config_value(settings::CONFIG_INSTITUTION_VERKEY, &my_vk);
}

pub async fn setup_indy_env() -> String {
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
    settings::get_agency_client_mut().unwrap().disable_test_mode();

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

    create_wallet(&config_wallet).unwrap();
    open_as_main_wallet(&config_wallet).unwrap();

    let config_issuer = configure_issuer_wallet(enterprise_seed).unwrap();
    init_issuer_config(&config_issuer).unwrap();

    provision_cloud_agent(&config_provision_agent).await.unwrap();

    settings::set_config_value(settings::CONFIG_GENESIS_PATH, utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH).to_str().unwrap());
    open_test_pool();

    let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
    institution_did
}

pub fn cleanup_indy_env() {
    delete_test_pool();
}

pub fn cleanup_agency_env() {
    delete_test_pool();
}

pub fn setup_agency_env() {
    debug!("setup_agency_env >> clearing up settings");
    settings::clear_config();

    settings::set_config_value(settings::CONFIG_GENESIS_PATH, utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH).to_str().unwrap());
    open_test_pool();
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
