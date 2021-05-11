use std::fs;
use std::sync::Once;

use indy::future::Future;
use indy::WalletHandle;
use rand::Rng;
use serde_json::Value;

use crate::{api, init, libindy, settings, utils};
use crate::agency_client::mocking::AgencyMockDecrypted;
use crate::libindy::utils::pool::reset_pool_handle;
use crate::libindy::utils::pool::tests::{create_test_ledger_config, delete_test_pool, open_test_pool};
use crate::libindy::utils::wallet::{close_main_wallet, create_indy_wallet, delete_wallet, reset_wallet_handle, WalletConfig, create_and_open_as_main_wallet, IssuerConfig, configure_issuer_wallet};
use crate::libindy::utils::wallet;
use crate::settings::set_testing_defaults;
use crate::utils::{get_temp_dir_path, runtime};
use crate::utils::constants;
use crate::utils::file::write_file;
use crate::utils::logger::LibvcxDefaultLogger;
use crate::utils::object_cache::ObjectCache;
use crate::utils::plugins::init_plugin;
use crate::utils::runtime::ThreadpoolConfig;
use crate::init::PoolConfig;
use crate::error::VcxErrorKind::WalletAccessFailed;
use crate::utils::provision::AgencyConfig;
use crate::utils::devsetup_agent::test::{Faber, Alice};
use crate::error::{VcxResult, VcxError};

pub struct SetupEmpty; // clears settings, setups up logging

pub struct SetupDefaults; // set default settings

pub struct SetupMocks; // set default settings and enable test mode

pub struct SetupIndyMocks; // set default settings and enable indy mode

pub struct SetupWallet {
    pub wallet_name: String,
    pub wallet_key: String,
    pub wallet_kdf: String,
    pub wallet_config: WalletConfig,
    skip_cleanup: bool,
} // creates wallet with random name, configures wallet settings

pub struct SetupPoolConfig {
    skip_cleanup: bool,
    pub pool_config: PoolConfig
}

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


fn setup(config: ThreadpoolConfig) {
    init_test_logging();
    settings::clear_config();
    set_testing_defaults();
    runtime::init_runtime(config);
}

fn setup_empty() {
    settings::clear_config();
    runtime::init_runtime(ThreadpoolConfig { num_threads: Some(4) });
    init_test_logging();
}

fn tear_down() {
    settings::clear_config();
    reset_wallet_handle();
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
        setup(ThreadpoolConfig { num_threads: Some(4) });
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
        setup(ThreadpoolConfig { num_threads: Some(4) });
        SetupMocks::_init()

    }

    pub fn init_without_threadpool() -> SetupMocks {
        setup(ThreadpoolConfig { num_threads: Some(0) });
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
        setup(ThreadpoolConfig { num_threads: Some(4) });
        let wallet_name: String = format!("Test_SetupLibraryWallet_{}", uuid::Uuid::new_v4().to_string());
        let wallet_key: String = settings::DEFAULT_WALLET_KEY.into();
        let wallet_kdf: String = settings::WALLET_KDF_RAW.into();
        settings::set_config_value(settings::CONFIG_WALLET_NAME, &wallet_name);
        settings::set_config_value(settings::CONFIG_WALLET_KEY, &wallet_key);
        settings::set_config_value(settings::CONFIG_WALLET_KEY_DERIVATION, &wallet_kdf);
        let wallet_config = WalletConfig {
            wallet_name: wallet_name.clone(),
            wallet_key: wallet_key.clone(),
            wallet_key_derivation: wallet_kdf.to_string(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None
        };

        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
        settings::get_agency_client_mut().unwrap().disable_test_mode();
        create_and_open_as_main_wallet(&wallet_config).unwrap();
        SetupLibraryWallet { wallet_name, wallet_key, wallet_kdf }
    }
}

impl Drop for SetupLibraryWallet {
    fn drop(&mut self) {
        let _res = close_main_wallet();
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
        settings::set_config_value(settings::CONFIG_WALLET_BACKUP_KEY, settings::DEFAULT_WALLET_BACKUP_KEY);
        settings::get_agency_client_mut().unwrap().disable_test_mode();

        let wallet_config = WalletConfig {
            wallet_name: wallet_name.clone(),
            wallet_key: wallet_key.clone(),
            wallet_key_derivation: wallet_kdf.clone(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None
        };
        create_indy_wallet(&wallet_config).unwrap();
        info!("SetupWallet:: init :: Wallet {} created", wallet_name);

        SetupWallet { wallet_name, wallet_kdf, wallet_key, wallet_config, skip_cleanup: false}
    }

    pub fn skip_cleanup(mut self) -> SetupWallet {
        self.skip_cleanup = true;
        self
    }
}

impl Drop for SetupWallet {
    fn drop(&mut self) {
        if self.skip_cleanup == false {
            let _res = close_main_wallet();
            delete_wallet(&self.wallet_name, &self.wallet_key, &self.wallet_kdf, None, None, None).unwrap();
            reset_wallet_handle();
        }
    }
}

impl SetupPoolConfig {
    pub fn init() -> SetupPoolConfig {
        create_test_ledger_config();
        let genesis_path = utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH).to_str().unwrap().to_string();
        settings::set_config_value(settings::CONFIG_GENESIS_PATH, &genesis_path.clone());
        let pool_config = PoolConfig {
            genesis_path,
            pool_name: None,
            pool_config: None
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
        setup(ThreadpoolConfig { num_threads: Some(4) });
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

impl SetupLibraryWalletPool {
    pub fn init() -> SetupLibraryWalletPool {
        setup(ThreadpoolConfig { num_threads: Some(4) });
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
        setup(ThreadpoolConfig { num_threads: Some(4) });
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
        setup(ThreadpoolConfig { num_threads: Some(4) });
        let wallet_name: String = format!("Test_SetupWalletAndPool_{}", uuid::Uuid::new_v4().to_string());
        let wallet_key: String = settings::DEFAULT_WALLET_KEY.into();
        let wallet_kdf: String = settings::WALLET_KDF_RAW.into();
        settings::set_config_value(settings::CONFIG_WALLET_NAME, &wallet_name);
        settings::set_config_value(settings::CONFIG_WALLET_KEY, &wallet_key);
        settings::set_config_value(settings::CONFIG_WALLET_KEY_DERIVATION, &wallet_kdf);
        settings::get_agency_client_mut().unwrap().enable_test_mode();

        let wallet_config = WalletConfig {
            wallet_name: wallet_name.clone(),
            wallet_key: wallet_key.clone(),
            wallet_key_derivation: wallet_kdf.to_string(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None
        };
        create_and_open_as_main_wallet(&wallet_config).unwrap();

        SetupAgencyMock { wallet_name, wallet_key, wallet_kdf }
    }
}

impl Drop for SetupAgencyMock {
    fn drop(&mut self) {
        let _res = close_main_wallet();
        delete_wallet(&self.wallet_name, &self.wallet_key, &self.wallet_kdf, None, None, None).unwrap();
        tear_down()
    }
}

impl SetupLibraryAgencyV2 {
    pub fn init() -> SetupLibraryAgencyV2 {
        setup(ThreadpoolConfig { num_threads: Some(4) });
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
        setup(ThreadpoolConfig { num_threads: Some(4) });
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
    libindy::utils::anoncreds::libindy_prover_create_master_secret(settings::DEFAULT_LINK_SECRET_ALIAS).unwrap();
    let (my_did, my_vk) = libindy::utils::signus::create_and_store_my_did(Some(constants::TRUSTEE_SEED), None).unwrap();
    settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &my_did);
    settings::set_config_value(settings::CONFIG_INSTITUTION_VERKEY, &my_vk);
}

pub fn setup_libnullpay_nofees() {
    init_plugin(settings::DEFAULT_PAYMENT_PLUGIN, settings::DEFAULT_PAYMENT_INIT_FUNCTION);
    libindy::utils::payments::tests::token_setup(None, None, true);
}

pub fn setup_indy_env(use_zero_fees: bool) {
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
    settings::get_agency_client_mut().unwrap().disable_test_mode();

    init_plugin(settings::DEFAULT_PAYMENT_PLUGIN, settings::DEFAULT_PAYMENT_INIT_FUNCTION);

    settings::set_config_value(settings::CONFIG_WALLET_KEY_DERIVATION, settings::WALLET_KDF_RAW);
    let wallet_config = WalletConfig {
        wallet_name: settings::DEFAULT_WALLET_NAME.into(),
        wallet_key: settings::DEFAULT_WALLET_KEY.into(),
        wallet_key_derivation: settings::WALLET_KDF_RAW.into(),
        wallet_type: None,
        storage_config: None,
        storage_credentials: None,
        rekey: None,
        rekey_derivation_method: None
    };
    create_and_open_as_main_wallet(&wallet_config).unwrap();

    settings::set_config_value(settings::CONFIG_GENESIS_PATH, utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH).to_str().unwrap());
    open_test_pool();

    libindy::utils::anoncreds::libindy_prover_create_master_secret(settings::DEFAULT_LINK_SECRET_ALIAS).ok();

    let (my_did, my_vk) = libindy::utils::signus::create_and_store_my_did(Some(constants::TRUSTEE_SEED), None).unwrap();
    settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &my_did);
    settings::set_config_value(settings::CONFIG_INSTITUTION_VERKEY, &my_vk);

    libindy::utils::payments::tests::token_setup(None, None, use_zero_fees);
}

pub fn cleanup_indy_env() {
    let _res = close_main_wallet();
    delete_wallet(settings::DEFAULT_WALLET_NAME, settings::DEFAULT_WALLET_KEY, settings::WALLET_KDF_RAW, None, None, None)
        .unwrap_or_else(|_| error!("Error deleting wallet {}", settings::DEFAULT_WALLET_NAME));
    delete_test_pool();
}

pub fn cleanup_agency_env() {
    delete_test_pool();
}

pub fn set_institution(institution_handle: u32) -> VcxResult<()> {
    CONFIG_STRING.get(institution_handle, |t| {
        set_new_config(t)
    })?;
    Ok(())
}

pub fn set_consumer(consumer_handle: u32) -> VcxResult<()> {
    CONFIG_STRING.get(consumer_handle, |t| {
        set_new_config(t)
    })?;
    Ok(())
}

pub fn set_institution_default() -> VcxResult<()> {
    unsafe {
        CONFIG_STRING.get(INSTITUTION_CONFIG, |t| {
            set_new_config(t)
        })?;
    }
    Ok(())
}

pub fn set_consumer_default() -> VcxResult<()> {
    unsafe {
        CONFIG_STRING.get(CONSUMER_CONFIG, |t| {
            set_new_config(t)
        })?;
    }
    Ok(())
}

pub fn set_new_config(config: &str) -> VcxResult<()> {
    settings::clear_config();
    settings::process_config_string(config, true)?;
    change_wallet_handle();
    Ok(())
}

fn change_wallet_handle() {
    warn!("change_wallet_handle >>> getting wallet_handle");
    let wallet_handle = settings::get_config_value(settings::CONFIG_WALLET_HANDLE).unwrap();
    warn!("change_wallet_handle >>> going to set wallet_handle={}", wallet_handle);
    wallet::set_wallet_handle(WalletHandle(wallet_handle.parse::<i32>().unwrap()));
    warn!("change_wallet_handle >>> finished setting wallet_handle={}", wallet_handle);
}

pub fn setup_agency_env(use_zero_fees: bool) {
    debug!("setup_agency_env >> clearing up settings");
    settings::clear_config();

    init_plugin(settings::DEFAULT_PAYMENT_PLUGIN, settings::DEFAULT_PAYMENT_INIT_FUNCTION);

    settings::set_config_value(settings::CONFIG_GENESIS_PATH, utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH).to_str().unwrap());
    open_test_pool();
}

pub fn combine_configs(wallet_config: &WalletConfig, agency_config: &AgencyConfig, issuer_config: Option<&IssuerConfig>, wallet_handle: WalletHandle, institution_name: Option<&str>) -> String {
    let wallet_config = serde_json::to_string(wallet_config).unwrap();
    let agency_config = serde_json::to_string(agency_config).unwrap();
    fn merge(a: &mut Value, b: &Value) {
        match (a, b) {
            (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
                for (k, v) in b {
                    merge(a.entry(k.clone()).or_insert(serde_json::Value::Null), v);
                }
            }
            (a, b) => {
                *a = b.clone();
            }
        }
    }

    let mut final_config: Value = serde_json::from_str(&wallet_config).unwrap();
    let agency_config: Value = serde_json::from_str(&agency_config).unwrap();
    merge(&mut final_config, &agency_config);

    if let Some(issuer_config) = issuer_config {
        let issuer_config = serde_json::to_string(issuer_config).unwrap();
        let mut issuer_config = serde_json::from_str::<serde_json::Value>(&issuer_config).unwrap();
        issuer_config[settings::CONFIG_INSTITUTION_NAME] = json!(institution_name.expect("Specified institution config, but not institution_name").to_string());
        merge(&mut final_config, &issuer_config);
    }

    final_config[settings::CONFIG_WALLET_HANDLE] = json!(wallet_handle.0.to_string());

    final_config.to_string()
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
    use crate::connection;

    use super::*;

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    pub fn test_two_enterprise_connections() {
        let _setup = SetupLibraryAgencyV2ZeroFees::init();
        let institution = Faber::setup();
        let consumer1 = Alice::setup();

        let (_faber, _alice) = connection::tests::create_connected_connections(&consumer1, &institution);
        let (_faber, _alice) = connection::tests::create_connected_connections(&consumer1, &institution);
    }
}
