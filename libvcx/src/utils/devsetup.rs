use std::fs;
use std::sync::Once;

use futures::Future;
use indy::WalletHandle;
use rand::Rng;
use serde_json::Value;

use ::{indy, init};
use ::{settings, utils};
use agency_comm::agency_settings;
use agency_comm::mocking::AgencyMockDecrypted;
use libindy::utils::pool::reset_pool_handle;
use libindy::utils::pool::tests::{create_test_ledger_config, delete_test_pool, open_test_pool};
use libindy::utils::wallet::{close_main_wallet, create_wallet, delete_wallet, reset_wallet_handle};
use libindy::utils::wallet;
use libindy::utils::wallet::create_and_open_as_main_wallet;
use settings::set_testing_defaults;
use utils::{get_temp_dir_path, threadpool};
use utils::constants;
use utils::file::write_file;
use utils::logger::LibvcxDefaultLogger;
use utils::object_cache::ObjectCache;
use utils::plugins::init_plugin;

pub struct SetupEmpty; // clears settings, setups up logging

pub struct SetupDefaults; // set default settings

pub struct SetupMocks; // set default settings and enable test mode

pub struct SetupIndyMocks; // set default settings and enable indy mode

pub struct SetupWallet {
    pub wallet_name: String,
    pub wallet_key: String,
    pub wallet_kdf: String,
} // creates wallet with random name, configures wallet settings

pub struct SetupPoolConfig;

pub struct SetupLibraryWallet {
    pub wallet_name: String,
    pub wallet_key: String,
    pub wallet_kdf: String,
} // set default settings and init indy wallet

pub struct SetupLibraryWalletPool; // set default settings, init indy wallet, init pool, set default fees

pub struct SetupLibraryWalletPoolZeroFees;  // set default settings, init indy wallet, init pool, set zero fees

pub struct SetupAgencyMock {
    pub wallet_name: String,
    pub wallet_key: String,
    pub wallet_kdf: String,
} // set default settings and enable mock agency mode

pub struct SetupLibraryAgencyV2; // init indy wallet, init pool, provision 2 agents. use protocol type 2.0

pub struct SetupLibraryAgencyV2ZeroFees; // init indy wallet, init pool, provision 2 agents. use protocol type 2.0, set zero fees

fn setup() {
    settings::clear_config();
    set_testing_defaults();
    threadpool::init();
    init_test_logging();
}

fn setup_empty() {
    settings::clear_config();
    threadpool::init();
    init_test_logging();
}

fn tear_down() {
    settings::clear_config();
    reset_wallet_handle();
    reset_pool_handle();
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
    pub fn init() -> SetupMocks {
        setup();
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
        ::std::env::set_var("DUMMY_TEST_MODE", "true");
        agency_settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
        SetupMocks
    }
}

impl Drop for SetupMocks {
    fn drop(&mut self) {
        ::std::env::set_var("DUMMY_TEST_MODE", "false");
        tear_down()
    }
}

impl SetupLibraryWallet {
    pub fn init() -> SetupLibraryWallet {
        setup();
        let wallet_name: String = format!("Test_SetupLibraryWallet_{}", uuid::Uuid::new_v4().to_string());
        let wallet_key: String = settings::DEFAULT_WALLET_KEY.into();
        let wallet_kdf: String = settings::WALLET_KDF_RAW.into();
        settings::set_config_value(settings::CONFIG_WALLET_NAME, &wallet_name);
        settings::set_config_value(settings::CONFIG_WALLET_KEY, &wallet_key);
        settings::set_config_value(settings::CONFIG_WALLET_KEY_DERIVATION, &wallet_kdf);

        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
        agency_settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
        create_and_open_as_main_wallet(&wallet_name, settings::DEFAULT_WALLET_KEY, settings::WALLET_KDF_RAW, None, None, None).unwrap();
        SetupLibraryWallet { wallet_name, wallet_key, wallet_kdf }
    }
}

impl Drop for SetupLibraryWallet {
    fn drop(&mut self) {
        close_main_wallet();
        delete_wallet(&self.wallet_name, &self.wallet_key, &self.wallet_kdf, None, None, None).unwrap();
        tear_down()
    }
}

impl SetupWallet {
    pub fn init() -> SetupWallet {
        init_test_logging();
        let wallet_name: String = format!("Test_SetupWallet_{}", uuid::Uuid::new_v4().to_string());
        let wallet_key: String = settings::DEFAULT_WALLET_KEY.into();
        let wallet_kdf: String = settings::WALLET_KDF_RAW.into();
        settings::set_config_value(settings::CONFIG_WALLET_NAME, &wallet_name);
        settings::set_config_value(settings::CONFIG_WALLET_KEY, &wallet_key);
        settings::set_config_value(settings::CONFIG_WALLET_KEY_DERIVATION, &wallet_kdf);

        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
        agency_settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
        settings::set_config_value(settings::CONFIG_WALLET_BACKUP_KEY, settings::DEFAULT_WALLET_BACKUP_KEY);

        create_wallet(&wallet_name, &wallet_key, &wallet_kdf, None, None, None).unwrap();
        info!("SetupWallet:: init :: Wallet {} created", wallet_name);

        SetupWallet { wallet_name, wallet_key, wallet_kdf }
    }
}

impl Drop for SetupWallet {
    fn drop(&mut self) {
        close_main_wallet();
        delete_wallet(&self.wallet_name, &self.wallet_key, &self.wallet_kdf, None, None, None).unwrap();
        reset_wallet_handle();
    }
}

impl SetupPoolConfig {
    pub fn init() -> SetupPoolConfig {
        create_test_ledger_config();
        settings::set_config_value(settings::CONFIG_GENESIS_PATH, utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH).to_str().unwrap());

        SetupPoolConfig {}
    }
}

impl Drop for SetupPoolConfig {
    fn drop(&mut self) {
        delete_test_pool();
        reset_pool_handle();
    }
}

impl SetupIndyMocks {
    pub fn init() -> SetupIndyMocks {
        setup();
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
        agency_settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
        SetupIndyMocks {}
    }
}

impl Drop for SetupIndyMocks {
    fn drop(&mut self) {
        tear_down()
    }
}

impl SetupLibraryWalletPool {
    pub fn init() -> SetupLibraryWalletPool {
        setup();
        setup_indy_env(false);
        SetupLibraryWalletPool
    }
}

impl Drop for SetupLibraryWalletPool {
    fn drop(&mut self) {
        cleanup_indy_env();
        tear_down()
    }
}

impl SetupLibraryWalletPoolZeroFees {
    pub fn init() -> SetupLibraryWalletPoolZeroFees {
        setup();
        setup_indy_env(true);
        SetupLibraryWalletPoolZeroFees
    }
}

impl Drop for SetupLibraryWalletPoolZeroFees {
    fn drop(&mut self) {
        cleanup_indy_env();
        tear_down()
    }
}

impl SetupAgencyMock {
    pub fn init() -> SetupAgencyMock {
        setup();
        let wallet_name: String = format!("Test_SetupWalletAndPool_{}", uuid::Uuid::new_v4().to_string());
        let wallet_key: String = settings::DEFAULT_WALLET_KEY.into();
        let wallet_kdf: String = settings::WALLET_KDF_RAW.into();
        settings::set_config_value(settings::CONFIG_WALLET_NAME, &wallet_name);
        settings::set_config_value(settings::CONFIG_WALLET_KEY, &wallet_key);
        settings::set_config_value(settings::CONFIG_WALLET_KEY_DERIVATION, &wallet_kdf);
        agency_settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "agency");
        create_and_open_as_main_wallet(&wallet_name, settings::DEFAULT_WALLET_KEY, settings::WALLET_KDF_RAW, None, None, None).unwrap();

        SetupAgencyMock { wallet_name, wallet_key, wallet_kdf }
    }
}

impl Drop for SetupAgencyMock {
    fn drop(&mut self) {
        close_main_wallet();
        delete_wallet(&self.wallet_name, &self.wallet_key, &self.wallet_kdf, None, None, None).unwrap();
        tear_down()
    }
}

impl SetupLibraryAgencyV2 {
    pub fn init() -> SetupLibraryAgencyV2 {
        setup();
        debug!("SetupLibraryAgencyV2 init >> going to setup agency environment");
        setup_agency_env(false);
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

impl SetupLibraryAgencyV2ZeroFees {
    pub fn init() -> SetupLibraryAgencyV2ZeroFees {
        setup();
        setup_agency_env(true);
        SetupLibraryAgencyV2ZeroFees
    }
}

impl Drop for SetupLibraryAgencyV2ZeroFees {
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

// TODO: We could have an array of configs
static mut INSTITUTION_CONFIG: u32 = 0;
static mut CONSUMER_CONFIG: u32 = 0;
// static mut CONFIGS: Vec<u32> = Vec::new(); // Vector of handles

lazy_static! {
    static ref CONFIG_STRING: ObjectCache<String> = ObjectCache::<String>::new("devsetup-config-cache");
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
    ::libindy::utils::anoncreds::libindy_prover_create_master_secret(settings::DEFAULT_LINK_SECRET_ALIAS).unwrap();
    let (my_did, my_vk) = ::libindy::utils::signus::create_and_store_my_did(Some(constants::TRUSTEE_SEED), None).unwrap();
    settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &my_did);
    settings::set_config_value(settings::CONFIG_INSTITUTION_VERKEY, &my_vk);
}

pub fn setup_libnullpay_nofees() {
    init_plugin(settings::DEFAULT_PAYMENT_PLUGIN, settings::DEFAULT_PAYMENT_INIT_FUNCTION);
    ::libindy::utils::payments::tests::token_setup(None, None, true);
}

pub fn setup_indy_env(use_zero_fees: bool) {
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
    agency_settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");

    init_plugin(settings::DEFAULT_PAYMENT_PLUGIN, settings::DEFAULT_PAYMENT_INIT_FUNCTION);

    settings::set_config_value(settings::CONFIG_WALLET_KEY_DERIVATION, settings::WALLET_KDF_RAW);
    create_and_open_as_main_wallet(settings::DEFAULT_WALLET_NAME, settings::DEFAULT_WALLET_KEY, settings::WALLET_KDF_RAW, None, None, None).unwrap();

    settings::set_config_value(settings::CONFIG_GENESIS_PATH, utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH).to_str().unwrap());
    open_test_pool();

    ::libindy::utils::anoncreds::libindy_prover_create_master_secret(settings::DEFAULT_LINK_SECRET_ALIAS).unwrap();

    let (my_did, my_vk) = ::libindy::utils::signus::create_and_store_my_did(Some(constants::TRUSTEE_SEED), None).unwrap();
    settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &my_did);
    settings::set_config_value(settings::CONFIG_INSTITUTION_VERKEY, &my_vk);

    ::libindy::utils::payments::tests::token_setup(None, None, use_zero_fees);
}

pub fn cleanup_indy_env() {
    close_main_wallet();
    delete_wallet(settings::DEFAULT_WALLET_NAME, settings::DEFAULT_WALLET_KEY, settings::WALLET_KDF_RAW, None, None, None)
        .unwrap_or_else(|_| error!("Error deleting wallet {}", settings::DEFAULT_WALLET_NAME));
    delete_test_pool();
}

pub fn cleanup_agency_env() {
    set_institution(None);
    close_main_wallet();
    delete_wallet(&settings::get_wallet_name().unwrap(), settings::DEFAULT_WALLET_KEY, settings::WALLET_KDF_RAW, None, None, None).unwrap();

    set_consumer(None);
    close_main_wallet();
    delete_wallet(&settings::get_wallet_name().unwrap(), settings::DEFAULT_WALLET_KEY, settings::WALLET_KDF_RAW, None, None, None).unwrap();

    delete_test_pool();
}

pub fn set_institution(institution_handle: Option<u32>) {
    settings::clear_config();
    unsafe {
        CONFIG_STRING.get(institution_handle.unwrap_or(INSTITUTION_CONFIG), |t| {
            settings::set_config_value(settings::CONFIG_PAYMENT_METHOD, settings::DEFAULT_PAYMENT_METHOD);
            let result = settings::process_config_string(&t, true);
            warn!("Switching test context to institution. Settings: {:?}", settings::settings_as_string());
            result
        }).unwrap();
    }
    change_wallet_handle();
}

pub fn set_consumer(consumer_handle: Option<u32>) {
    settings::clear_config();
    info!("Using consumer handle: {:?}", consumer_handle);
    unsafe {
        CONFIG_STRING.get(consumer_handle.unwrap_or(CONSUMER_CONFIG), |t| {
            warn!("Setting consumer config {}", t);
            settings::set_config_value(settings::CONFIG_PAYMENT_METHOD, settings::DEFAULT_PAYMENT_METHOD);
            let result = settings::process_config_string(&t, true);
            warn!("Switching test context to consumer. Settings: {:?}", settings::settings_as_string());
            result
        }).unwrap();
    }
    change_wallet_handle();
}

fn change_wallet_handle() {
    let wallet_handle = settings::get_config_value(settings::CONFIG_WALLET_HANDLE).unwrap();
    unsafe { wallet::WALLET_HANDLE = WalletHandle(wallet_handle.parse::<i32>().unwrap()) }
}

fn assign_trustee_role(institution_handle: Option<u32>) {
    set_institution(institution_handle);
    let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
    let vk = settings::get_config_value(settings::CONFIG_INSTITUTION_VERKEY).unwrap();
    settings::clear_config();

    wallet::create_and_open_as_main_wallet(settings::DEFAULT_WALLET_NAME, settings::DEFAULT_WALLET_KEY, settings::WALLET_KDF_RAW, None, None, None).unwrap();
    let (trustee_did, _) = ::libindy::utils::signus::create_and_store_my_did(Some(constants::TRUSTEE_SEED), None).unwrap();
    let req_nym = ::indy::ledger::build_nym_request(&trustee_did, &did, Some(&vk), None, Some("TRUSTEE")).wait().unwrap();
    ::libindy::utils::ledger::libindy_sign_and_submit_request(&trustee_did, &req_nym).unwrap();

    close_main_wallet();
    wallet::delete_wallet(settings::DEFAULT_WALLET_NAME, settings::DEFAULT_WALLET_KEY, settings::WALLET_KDF_RAW, None, None, None).unwrap();
}

pub fn setup_agency_env(use_zero_fees: bool) {
    debug!("setup_agency_env >> clearing up settings");
    settings::clear_config();

    init_plugin(settings::DEFAULT_PAYMENT_PLUGIN, settings::DEFAULT_PAYMENT_INIT_FUNCTION);

    let enterprise_wallet_name = format!("{}_{}", constants::ENTERPRISE_PREFIX, settings::DEFAULT_WALLET_NAME);

    let seed1 = create_new_seed();
    let mut config = json!({
            "agency_url": AGENCY_ENDPOINT.to_string(),
            "agency_did": AGENCY_DID.to_string(),
            "agency_verkey": AGENCY_VERKEY.to_string(),
            "wallet_name": enterprise_wallet_name,
            "wallet_key": settings::DEFAULT_WALLET_KEY.to_string(),
            "wallet_key_derivation": settings::WALLET_KDF_RAW,
            "enterprise_seed": seed1,
            "agent_seed": seed1,
            "name": "institution".to_string(),
            "logo": "http://www.logo.com".to_string(),
            "path": constants::GENESIS_PATH.to_string()
        });

    debug!("setup_agency_env >> Going to provision enterprise using config: {:?}", &config);
    let enterprise_config = ::utils::provision::connect_register_provision(&config.to_string()).unwrap();

    ::api::vcx::vcx_shutdown(false);

    let consumer_wallet_name = format!("{}_{}", constants::CONSUMER_PREFIX, settings::DEFAULT_WALLET_NAME);
    let seed2 = create_new_seed();
    let mut config = json!({
            "agency_url": C_AGENCY_ENDPOINT.to_string(),
            "agency_did": C_AGENCY_DID.to_string(),
            "agency_verkey": C_AGENCY_VERKEY.to_string(),
            "wallet_name": consumer_wallet_name,
            "wallet_key": settings::DEFAULT_WALLET_KEY.to_string(),
            "wallet_key_derivation": settings::WALLET_KDF_RAW.to_string(),
            "enterprise_seed": seed2,
            "agent_seed": seed2,
            "name": "consumer".to_string(),
            "logo": "http://www.logo.com".to_string(),
            "path": constants::GENESIS_PATH.to_string()
        });

    debug!("setup_agency_env >> Going to provision consumer using config: {:?}", &config);
    let consumer_config = ::utils::provision::connect_register_provision(&config.to_string()).unwrap();

    unsafe {
        INSTITUTION_CONFIG = CONFIG_STRING.add(config_with_wallet_handle(&enterprise_wallet_name, &enterprise_config)).unwrap();
    }
    unsafe {
        CONSUMER_CONFIG = CONFIG_STRING.add(config_with_wallet_handle(&consumer_wallet_name, &consumer_config.to_string())).unwrap();
    }
    settings::set_config_value(settings::CONFIG_GENESIS_PATH, utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH).to_str().unwrap());
    open_test_pool();

    assign_trustee_role(None);

    // as trustees, mint tokens into each wallet
    set_institution(None);
    ::libindy::utils::payments::tests::token_setup(None, None, use_zero_fees);
}

pub fn config_with_wallet_handle(wallet_n: &str, config: &str) -> String {
    let config_val: Value = serde_json::from_str(config).unwrap();
    let wallet_key = config_val["wallet_key"].as_str().unwrap();

    let wallet_handle = init::open_as_main_wallet(wallet_n, wallet_key, settings::WALLET_KDF_RAW, None, None, None).unwrap();
    let mut config: serde_json::Value = serde_json::from_str(config).unwrap();
    config[settings::CONFIG_WALLET_HANDLE] = json!(wallet_handle.0.to_string());
    config.to_string()
}

pub fn create_consumer_config() -> u32 {
    settings::clear_config();
    let consumer_id: u32 = CONFIG_STRING.len().unwrap() as u32;
    let mut rng = rand::thread_rng();
    let random_salt = rng.gen::<u32>();
    let consumer_wallet_name = format!("{}_{}_{}", constants::CONSUMER_PREFIX, consumer_id, random_salt);
    let seed = create_new_seed();
    let config = json!({
            "agency_url": C_AGENCY_ENDPOINT.to_string(),
            "agency_did": C_AGENCY_DID.to_string(),
            "agency_verkey": C_AGENCY_VERKEY.to_string(),
            "wallet_name": consumer_wallet_name,
            "wallet_key": settings::DEFAULT_WALLET_KEY.to_string(),
            "wallet_key_derivation": settings::WALLET_KDF_RAW.to_string(),
            "enterprise_seed": seed,
            "agent_seed": seed,
            "name": format!("consumer_{}", consumer_id).to_string(),
            "logo": "http://www.logo.com".to_string(),
            "path": constants::GENESIS_PATH.to_string()
        });

    debug!("create_consumer_config >> Going to provision consumer using config: {:?}", &config);
    let consumer_config = ::utils::provision::connect_register_provision(&config.to_string()).unwrap();

    CONFIG_STRING.add(config_with_wallet_handle(&consumer_wallet_name, &consumer_config.to_string())).unwrap()
}

pub fn create_institution_config() -> u32 {
    settings::clear_config();

    let enterprise_id: u32 = CONFIG_STRING.len().unwrap() as u32;
    let mut rng = rand::thread_rng();
    let random_salt = rng.gen::<u32>();
    let enterprise_wallet_name = format!("{}_{}_{}", constants::ENTERPRISE_PREFIX, enterprise_id, random_salt);

    let seed = create_new_seed();
    let mut config = json!({
            "agency_url": AGENCY_ENDPOINT.to_string(),
            "agency_did": AGENCY_DID.to_string(),
            "agency_verkey": AGENCY_VERKEY.to_string(),
            "wallet_name": enterprise_wallet_name,
            "wallet_key": settings::DEFAULT_WALLET_KEY.to_string(),
            "wallet_key_derivation": settings::WALLET_KDF_RAW,
            "enterprise_seed": seed,
            "agent_seed": seed,
            "name": format!("institution_{}", enterprise_id).to_string(),
            "logo": "http://www.logo.com".to_string(),
            "path": constants::GENESIS_PATH.to_string()
        });

    debug!("create_institution_config >> Going to provision enterprise using config: {:?}", &config);
    let enterprise_config = ::utils::provision::connect_register_provision(&config.to_string()).unwrap();

    let handle = CONFIG_STRING.add(config_with_wallet_handle(&enterprise_wallet_name, &enterprise_config.to_string())).unwrap();

    assign_trustee_role(Some(handle));

    handle
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

#[cfg(feature = "agency_pool_tests")]
mod tests {
    use super::*;

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    pub fn test_two_enterprise_connections() {
        let _setup = SetupLibraryAgencyV2ZeroFees::init();

        let (_faber, _alice) = ::connection::tests::create_connected_connections(None, None);
        let (_faber, _alice) = ::connection::tests::create_connected_connections(None, None);
    }
}
