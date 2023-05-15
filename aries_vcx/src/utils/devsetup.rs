#![allow(clippy::unwrap_used)]

use std::fs;
use std::future::Future;
use std::sync::{Arc, Once};

use aries_vcx_core::global::settings::{
    disable_indy_mocks as disable_indy_mocks_core, enable_indy_mocks as enable_indy_mocks_core,
};
use aries_vcx_core::indy::ledger::pool::test_utils::{create_test_ledger_config, delete_test_pool, open_test_pool};
use aries_vcx_core::indy::ledger::pool::PoolConfig;
use aries_vcx_core::indy::utils::mocks::did_mocks::DidMocks;
use aries_vcx_core::indy::utils::mocks::pool_mocks::PoolMocks;
use aries_vcx_core::indy::wallet::{
    close_wallet, create_and_open_wallet, create_indy_wallet, create_wallet_with_master_secret, delete_wallet,
    open_wallet, wallet_configure_issuer, WalletConfig,
};

#[cfg(feature = "modular_libs")]
use aries_vcx_core::ledger::request_submitter::vdr_ledger::LedgerPoolConfig;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use aries_vcx_core::wallet::indy_wallet::IndySdkWallet;
use aries_vcx_core::{PoolHandle, WalletHandle};
use chrono::{DateTime, Duration, Utc};

use futures::future::BoxFuture;

use agency_client::agency_client::AgencyClient;
use agency_client::configuration::AgentProvisionConfig;
use agency_client::testing::mocking::{disable_agency_mocks, enable_agency_mocks, AgencyMockDecrypted};

#[cfg(feature = "modular_libs")]
use crate::core::profile::modular_libs_profile::ModularLibsProfile;
use crate::core::profile::profile::Profile;
#[cfg(feature = "vdrtools")]
use crate::core::profile::vdrtools_profile::VdrtoolsProfile;
use crate::global::settings;
use crate::global::settings::init_issuer_config;
use crate::global::settings::{aries_vcx_disable_indy_mocks, aries_vcx_enable_indy_mocks, set_test_configs};
use crate::utils;
use crate::utils::constants::GENESIS_PATH;
use crate::utils::file::write_file;
use crate::utils::get_temp_dir_path;
use crate::utils::provision::provision_cloud_agent;
use crate::utils::test_logger::LibvcxDefaultLogger;

pub struct SetupEmpty;

pub struct SetupDefaults;

pub struct SetupMocks {
    pub institution_did: String,
}

pub struct SetupIndyMocks;

pub struct TestSetupCreateWallet {
    pub wallet_config: WalletConfig,
    skip_cleanup: bool,
}

pub struct SetupPoolConfig {
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
    pub pool_handle: PoolHandle,
}

pub struct SetupWalletPool {
    pub institution_did: String,
    pub wallet_handle: WalletHandle,
    pub pool_handle: PoolHandle,
}

#[derive(Clone)]
pub struct SetupProfile {
    pub institution_did: String,
    pub profile: Arc<dyn Profile>,
    pub(self) teardown: Arc<dyn Fn() -> BoxFuture<'static, ()>>,
}

pub struct SetupInstitutionWallet {
    pub institution_did: String,
    pub wallet_handle: WalletHandle,
}

pub struct SetupPool {
    pub pool_handle: PoolHandle,
    pub genesis_file_path: String,
}

fn reset_global_state() {
    warn!("reset_global_state >>");
    AgencyMockDecrypted::clear_mocks();
    PoolMocks::clear_mocks();
    DidMocks::clear_mocks();
    aries_vcx_disable_indy_mocks().unwrap();
    disable_indy_mocks_core().unwrap();
    settings::reset_config_values().unwrap();
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
        aries_vcx_enable_indy_mocks().unwrap();
        enable_indy_mocks_core().unwrap();
        SetupMocks { institution_did }
    }
}

impl Drop for SetupMocks {
    fn drop(&mut self) {
        reset_global_state();
    }
}

impl SetupLibraryWallet {
    async fn init() -> SetupLibraryWallet {
        init_test_logging();

        debug!("SetupLibraryWallet::init >>");

        set_test_configs();

        let wallet_name: String = format!("Test_SetupLibraryWallet_{}", uuid::Uuid::new_v4());
        let wallet_key: String = settings::DEFAULT_WALLET_KEY.into();
        let wallet_kdf: String = settings::WALLET_KDF_RAW.into();
        let wallet_config = WalletConfig {
            wallet_name,
            wallet_key,
            wallet_key_derivation: wallet_kdf,
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

    pub async fn run<F>(f: impl FnOnce(Self) -> F)
    where
        F: Future<Output = ()>,
    {
        let init = Self::init().await;

        let handle = init.wallet_handle;
        let config = init.wallet_config.clone();

        f(init).await;

        close_wallet(handle).await.unwrap();

        delete_wallet(&config).await.unwrap();

        reset_global_state();
    }
}

impl TestSetupCreateWallet {
    async fn init() -> TestSetupCreateWallet {
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

    pub fn skip_cleanup(&mut self) -> &mut TestSetupCreateWallet {
        self.skip_cleanup = true;
        self
    }

    pub async fn run<F>(f: impl FnOnce(Self) -> F)
    where
        F: Future<Output = bool>,
    {
        let init = Self::init().await;

        let config = init.wallet_config.clone();

        let skip_cleanup = f(init).await;

        if skip_cleanup == false {
            delete_wallet(&config)
                .await
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

        SetupPoolConfig { pool_config }
    }
}

impl Drop for SetupPoolConfig {
    fn drop(&mut self) {
        reset_global_state();
    }
}

impl SetupIndyMocks {
    pub fn init() -> SetupIndyMocks {
        init_test_logging();
        aries_vcx_enable_indy_mocks().unwrap();
        enable_indy_mocks_core().unwrap();
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
        let pool_handle = open_test_pool().await;
        SetupWalletPoolAgency {
            agency_client,
            institution_did,
            wallet_handle,
            pool_handle,
        }
    }

    pub async fn run<F>(f: impl FnOnce(Self) -> F)
    where
        F: Future<Output = ()>,
    {
        let init = Self::init().await;

        let pool_handle = init.pool_handle;

        f(init).await;

        delete_test_pool(pool_handle).await;

        reset_global_state();
    }
}

impl SetupWalletPool {
    async fn init() -> SetupWalletPool {
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
            pool_handle,
        }
    }

    pub async fn run<F>(f: impl FnOnce(Self) -> F)
    where
        F: Future<Output = ()>,
    {
        let init = Self::init().await;

        let pool_handle = init.pool_handle;

        f(init).await;

        delete_test_pool(pool_handle).await;

        reset_global_state();
    }
}

impl SetupProfile {
    pub(self) fn should_run_modular() -> bool {
        cfg!(feature = "modular_libs")
    }

    #[cfg(any(feature = "modular_libs", feature = "vdrtools"))]
    pub async fn init() -> SetupProfile {
        init_test_logging();
        set_test_configs();

        #[cfg(feature = "modular_libs")]
        return {
            info!("SetupProfile >> using modular profile");
            SetupProfile::init_modular().await
        };

        #[cfg(feature = "vdr_proxy_ledger")]
        return {
            info!("SetupProfile >> using vdr proxy profile");
            SetupProfile::init_vdr_proxy_ledger().await
        };

        #[cfg(feature = "vdrtools")]
        return {
            info!("SetupProfile >> using indy profile");
            SetupProfile::init_indy().await
        };
    }

    // FUTURE - ideally no tests should be using this method, they should be using the generic init
    // after modular profile Anoncreds/Ledger methods have all been implemented, all tests should use init()
    #[cfg(feature = "vdrtools")]
    async fn init_indy() -> SetupProfile {
        let (institution_did, wallet_handle) = setup_issuer_wallet().await;

        settings::set_config_value(
            settings::CONFIG_GENESIS_PATH,
            utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH)
                .to_str()
                .unwrap(),
        )
        .unwrap();
        let pool_handle = open_test_pool().await;

        let profile: Arc<dyn Profile> = Arc::new(VdrtoolsProfile::new(wallet_handle, pool_handle.clone()));

        async fn indy_teardown(pool_handle: PoolHandle) {
            delete_test_pool(pool_handle.clone()).await;
        }

        SetupProfile {
            institution_did,
            profile,
            teardown: Arc::new(move || Box::pin(indy_teardown(pool_handle))),
        }
    }

    #[cfg(feature = "modular_libs")]
    async fn init_modular() -> SetupProfile {
        use aries_vcx_core::indy::ledger::pool::test_utils::create_tmp_genesis_txn_file;

        let (institution_did, wallet_handle) = setup_issuer_wallet().await;

        let genesis_file_path = create_tmp_genesis_txn_file();

        let wallet = IndySdkWallet::new(wallet_handle);

        let profile: Arc<dyn Profile> =
            Arc::new(ModularLibsProfile::new(Arc::new(wallet), LedgerPoolConfig { genesis_file_path }).unwrap());

        Arc::clone(&profile)
            .inject_anoncreds()
            .prover_create_link_secret(settings::DEFAULT_LINK_SECRET_ALIAS)
            .await
            .unwrap();

        async fn modular_teardown() {
            // nothing to do
        }

        SetupProfile {
            institution_did,
            profile,
            teardown: Arc::new(move || Box::pin(modular_teardown())),
        }
    }

    #[cfg(feature = "vdr_proxy_ledger")]
    async fn init_vdr_proxy_ledger() -> SetupProfile {
        use std::env;

        use crate::core::profile::vdr_proxy_profile::VdrProxyProfile;
        use aries_vcx_core::VdrProxyClient;

        let (institution_did, wallet_handle) = setup_issuer_wallet().await;

        // TODO: Test configuration should be handled uniformly
        let client_url = env::var("VDR_PROXY_CLIENT_URL").unwrap_or_else(|_| "http://127.0.0.1:3030".to_string());
        let client = VdrProxyClient::new(&client_url).unwrap();

        let profile: Arc<dyn Profile> = Arc::new(VdrProxyProfile::new(wallet_handle, client));

        async fn vdr_proxy_teardown() {
            // nothing to do
        }

        SetupProfile {
            institution_did,
            profile,
            teardown: Arc::new(move || Box::pin(vdr_proxy_teardown())),
        }
    }

    #[cfg(any(feature = "modular_libs", feature = "vdrtools", feature = "vdr_proxy_ledger"))]
    pub async fn run<F>(f: impl FnOnce(Self) -> F)
    where
        F: Future<Output = ()>,
    {
        let init = Self::init().await;

        let teardown = Arc::clone(&init.teardown);

        f(init).await;

        (teardown)().await;

        reset_global_state();
    }

    // FUTURE - ideally no tests should be using this method, they should be using the generic run
    // after modular profile Anoncreds/Ledger methods have all been implemented, all tests should use run()
    #[cfg(any(feature = "vdrtools", feature = "vdr_proxy_ledger"))]
    pub async fn run_indy<F>(f: impl FnOnce(Self) -> F)
    where
        F: Future<Output = ()>,
    {
        #[cfg(feature = "vdr_proxy_ledger")]
        let init = Self::init_vdr_proxy_ledger().await;

        #[cfg(all(feature = "vdrtools", not(feature = "vdr_proxy_ledger")))]
        let init = Self::init_indy().await;

        let teardown = Arc::clone(&init.teardown);

        f(init).await;

        (teardown)().await;

        reset_global_state();
    }
}

// TODO - FUTURE - delete this method after `SetupProfile::run_indy` is removed. The purpose of this helper method
// is to return a test profile for a prover/holder given an existing indy-based profile setup (i.e. returned by SetupProfile::run_indy)
#[cfg(any(feature = "modular_libs", feature = "vdrtools"))]
pub async fn init_holder_setup_in_indy_context(indy_issuer_setup: &SetupProfile) -> SetupProfile {
    if SetupProfile::should_run_modular() {
        return SetupProfile::init().await; // create a new modular profile
    }
    indy_issuer_setup.clone() // if indy runtime, just re-use the issuer setup
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

impl SetupPool {
    async fn init() -> SetupPool {
        debug!("SetupPool init >> going to setup agency environment");
        init_test_logging();

        let genesis_file_path = utils::get_temp_dir_path(GENESIS_PATH).to_str().unwrap().to_string();
        settings::set_config_value(settings::CONFIG_GENESIS_PATH, &genesis_file_path).unwrap();

        let pool_handle = open_test_pool().await;

        debug!("SetupPool init >> completed");
        SetupPool {
            pool_handle,
            genesis_file_path,
        }
    }

    pub async fn run<F>(f: impl FnOnce(Self) -> F)
    where
        F: Future<Output = ()>,
    {
        let init = Self::init().await;

        let handle = init.pool_handle;

        f(init).await;

        delete_test_pool(handle).await;

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

pub const AGENCY_ENDPOINT: &str = "http://localhost:8080";
pub const AGENCY_DID: &str = "VsKV7grR1BUE29mG2Fm2kX";
pub const AGENCY_VERKEY: &str = "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR";

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
    format!("{x:032}")
}

pub async fn setup_issuer_wallet_and_agency_client() -> (String, WalletHandle, AgencyClient) {
    let enterprise_seed = "000000000000000000000000Trustee1";
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
    let config_provision_agent = AgentProvisionConfig {
        agency_did: AGENCY_DID.to_string(),
        agency_verkey: AGENCY_VERKEY.to_string(),
        agency_endpoint: AGENCY_ENDPOINT.parse().expect("valid url"),
        agent_seed: None,
    };
    create_wallet_with_master_secret(&config_wallet).await.unwrap();
    let wallet_handle = open_wallet(&config_wallet).await.unwrap();
    let config_issuer = wallet_configure_issuer(wallet_handle, enterprise_seed).await.unwrap();
    init_issuer_config(&config_issuer.institution_did).unwrap();
    let mut agency_client = AgencyClient::new();

    let wallet: Arc<dyn BaseWallet> = Arc::new(IndySdkWallet::new(wallet_handle));

    provision_cloud_agent(&mut agency_client, wallet, &config_provision_agent)
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
    init_issuer_config(&config_issuer.institution_did).unwrap();
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
    use super::*;
    use chrono::SecondsFormat;
    use std::ops::Sub;

    #[test]
    fn test_is_past_timestamp() {
        let now = Utc::now();
        let past1ms_rfc3339 = now
            .sub(Duration::milliseconds(1))
            .to_rfc3339_opts(SecondsFormat::Millis, true);
        assert!(was_in_past(&past1ms_rfc3339, Duration::milliseconds(10)).unwrap())
    }
}
