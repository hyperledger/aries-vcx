use std::fs;
use std::sync::Once;
use indy_sys::WalletHandle;
use agency_client::configuration::AgentProvisionConfig;

use agency_client::testing::mocking::AgencyMockDecrypted;

use crate::{global, libindy, utils};
use crate::global::agency_client::get_main_agency_client;
use crate::global::settings;
use crate::global::settings::init_issuer_config;
use crate::libindy::utils::pool::PoolConfig;
use crate::libindy::utils::mocks::did_mocks::DidMocks;
use crate::libindy::utils::mocks::pool_mocks::{enable_pool_mocks, PoolMocks};
use crate::global::pool::reset_main_pool_handle;
use crate::libindy::utils::pool::test_utils::{create_test_ledger_config, delete_test_pool, open_test_pool};
use crate::libindy::utils::wallet::{create_indy_wallet, delete_wallet, WalletConfig};
use crate::global::wallet::main_wallet_configure_issuer;
use crate::global::settings::{disable_indy_mocks, enable_indy_mocks, set_test_configs};
use crate::global::wallet::{close_main_wallet, create_and_open_as_main_wallet, create_main_wallet, open_as_main_wallet, reset_main_wallet_handle};
use crate::utils::constants;
use crate::utils::file::write_file;
use crate::utils::get_temp_dir_path;
use crate::utils::provision::provision_cloud_agent;
use crate::utils::test_logger::LibvcxDefaultLogger;

pub struct SetupEmpty; // clears settings, setups up logging

pub struct SetupDefaults; // set default settings

pub struct SetupMocks; // set default settings and enable test mode

pub struct SetupIndyMocks; // set default settings and enable indy mode

pub struct SetupPoolMocks {
    pub wallet_handle: WalletHandle
}

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
    pub wallet_handle: WalletHandle
} // set default settings and init indy wallet

pub struct SetupWithWalletAndAgency {
    pub institution_did: String,
    pub wallet_handle: WalletHandle
}  // set default settings, init indy wallet, init pool

pub struct SetupAgencyMock {
    pub wallet_config: WalletConfig,
} // set default settings and enable mock agency mode

pub struct SetupLibraryAgencyV2; // init indy wallet, init pool, provision 2 agents. use protocol type 2.0

fn reset_global_state() {
    warn!("reset_global_state >>");
    AgencyMockDecrypted::clear_mocks();
    PoolMocks::clear_mocks();
    DidMocks::clear_mocks();
    reset_main_wallet_handle().unwrap();
    reset_main_pool_handle();
    disable_indy_mocks();
    settings::reset_config_values();
    global::agency_client::reset_main_agency_client();
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
        set_test_configs();
        global::agency_client::get_main_agency_client_mut().unwrap().enable_test_mode();
        enable_indy_mocks();
        SetupMocks {} // todo: not needed since we don't implement drop
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

        let wallet_handle = create_and_open_as_main_wallet(&wallet_config).await.unwrap();
        SetupLibraryWallet { wallet_config, wallet_handle }
    }
}

impl Drop for SetupLibraryWallet {
    fn drop(&mut self) {
        let _res = futures::executor::block_on(close_main_wallet()).unwrap();
        futures::executor::block_on(delete_wallet(&self.wallet_config)).unwrap();
        reset_global_state();
    }
}

impl SetupWallet {
    pub async fn init() -> SetupWallet {
        init_test_logging();
        set_test_configs();
        let wallet_name: String = format!("Test_SetupWallet_{}", uuid::Uuid::new_v4().to_string());
        global::agency_client::get_main_agency_client_mut().unwrap().disable_test_mode();
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
            let _res = futures::executor::block_on(close_main_wallet()).unwrap_or_else(|_e| error!("Failed to close main wallet while dropping SetupWallet test config."));
            futures::executor::block_on(delete_wallet(&self.wallet_config)).unwrap_or_else(|_e| error!("Failed to delete wallet while dropping SetupWallet test config."));
        }
        reset_global_state();
    }
}

impl SetupPoolConfig {
    pub async fn init() -> SetupPoolConfig {
        init_test_logging();

        create_test_ledger_config().await;
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
            futures::executor::block_on(delete_test_pool());
            reset_main_pool_handle();
        }
        reset_global_state();
    }
}

impl SetupPoolMocks {
    pub async fn init() -> SetupPoolMocks {
        init_test_logging();
        let (_issuer_did, wallet_handle) = setup_indy_env().await;
        enable_pool_mocks();
        SetupPoolMocks {
            wallet_handle
        }
    }
}

impl Drop for SetupPoolMocks {
    fn drop(&mut self) {
        reset_global_state();
    }
}

impl SetupIndyMocks {
    pub fn init() -> SetupIndyMocks {
        init_test_logging();
        enable_indy_mocks();
        global::agency_client::get_main_agency_client_mut().unwrap().enable_test_mode();
        SetupIndyMocks {}
    }
}

impl Drop for SetupIndyMocks {
    fn drop(&mut self) {
        reset_global_state();
    }
}

impl SetupWithWalletAndAgency {
    pub async fn init() -> SetupWithWalletAndAgency {
        init_test_logging();
        set_test_configs();

        let (institution_did, wallet_handle) = setup_indy_env().await;

        settings::set_config_value(settings::CONFIG_GENESIS_PATH, utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH).to_str().unwrap());
        open_test_pool().await;
        SetupWithWalletAndAgency {
            institution_did,
            wallet_handle
        }
    }
}

impl Drop for SetupWithWalletAndAgency {
    fn drop(&mut self) {
        futures::executor::block_on(delete_test_pool());
        reset_global_state();
    }
}

impl SetupAgencyMock {
    pub async fn init() -> SetupAgencyMock {
        init_test_logging();

        let wallet_name: String = format!("Test_SetupWalletAndPool_{}", uuid::Uuid::new_v4().to_string());
        global::agency_client::get_main_agency_client_mut().unwrap().enable_test_mode();
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
        create_and_open_as_main_wallet(&wallet_config).await.unwrap();

        SetupAgencyMock { wallet_config }
    }
}

impl Drop for SetupAgencyMock {
    fn drop(&mut self) {
        let _res = futures::executor::block_on(close_main_wallet()).unwrap();
        futures::executor::block_on(delete_wallet(&self.wallet_config)).unwrap();
        reset_global_state();
    }
}

impl SetupLibraryAgencyV2 {
    pub async fn init() -> SetupLibraryAgencyV2 {
        debug!("SetupLibraryAgencyV2 init >> going to setup agency environment");
        init_test_logging();

        settings::set_config_value(settings::CONFIG_GENESIS_PATH, utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH).to_str().unwrap());
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
    ($pattern:pat, $var:expr) => (
        assert!(match $var {
            $pattern => true,
            _ => false
        })
    );
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

pub async fn configure_trustee_did(wallet_handle: WalletHandle) {
    libindy::utils::anoncreds::libindy_prover_create_master_secret(wallet_handle, settings::DEFAULT_LINK_SECRET_ALIAS).await.unwrap();
    let (my_did, my_vk) = libindy::utils::signus::create_and_store_my_did(wallet_handle, Some(constants::TRUSTEE_SEED), None).await.unwrap();
    settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &my_did);
    settings::set_config_value(settings::CONFIG_INSTITUTION_VERKEY, &my_vk);
}

pub async fn setup_indy_env() -> (String, WalletHandle) {
    global::agency_client::get_main_agency_client_mut().unwrap().disable_test_mode();

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

    create_main_wallet(&config_wallet).await.unwrap();
    let wallet_handle = open_as_main_wallet(&config_wallet).await.unwrap();

    let config_issuer = main_wallet_configure_issuer(enterprise_seed).await.unwrap();
    init_issuer_config(&config_issuer).unwrap();

    provision_cloud_agent(&mut get_main_agency_client().unwrap(), wallet_handle, &config_provision_agent).await.unwrap();

    let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
    (institution_did, wallet_handle)
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
