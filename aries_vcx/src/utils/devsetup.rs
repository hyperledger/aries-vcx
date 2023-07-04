#![allow(clippy::unwrap_used)]

use std::fs;
use std::future::Future;
use std::sync::{Arc, Once};

use aries_vcx_core::global::settings::{
    disable_indy_mocks as disable_indy_mocks_core, enable_indy_mocks as enable_indy_mocks_core,
    reset_config_values_ariesvcxcore,
};
use aries_vcx_core::indy::utils::mocks::did_mocks::DidMocks;
use aries_vcx_core::indy::utils::mocks::pool_mocks::PoolMocks;
use aries_vcx_core::indy::wallet::{
    create_wallet_with_master_secret, open_wallet, wallet_configure_issuer, WalletConfig,
};

#[cfg(feature = "modular_libs")]
use aries_vcx_core::ledger::request_submitter::vdr_ledger::LedgerPoolConfig;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use aries_vcx_core::wallet::indy_wallet::IndySdkWallet;
use aries_vcx_core::{PoolHandle, WalletHandle};
use chrono::{DateTime, Duration, Utc};

use futures::future::BoxFuture;
use uuid::Uuid;

use agency_client::agency_client::AgencyClient;
use agency_client::configuration::AgentProvisionConfig;
use agency_client::testing::mocking::{enable_agency_mocks, AgencyMockDecrypted};
use aries_vcx_core::indy::ledger::pool::test_utils::create_testpool_genesis_txn_file;
use aries_vcx_core::indy::ledger::pool::{
    create_pool_ledger_config, indy_close_pool, indy_delete_pool, indy_open_pool,
};

#[cfg(feature = "mixed_breed")]
use crate::core::profile::mixed_breed_profile::MixedBreedProfile;
#[cfg(feature = "modular_libs")]
use crate::core::profile::modular_libs_profile::ModularLibsProfile;
#[cfg(feature = "modular_libs")]
use crate::core::profile::prepare_taa_options;
use crate::core::profile::profile::Profile;
#[cfg(feature = "vdrtools")]
use crate::core::profile::vdrtools_profile::VdrtoolsProfile;
use crate::global::settings;
use crate::global::settings::{
    aries_vcx_disable_indy_mocks, aries_vcx_enable_indy_mocks, set_config_value, CONFIG_INSTITUTION_DID, DEFAULT_DID,
};
use crate::global::settings::{init_issuer_config, reset_config_values_ariesvcx};
use crate::utils;
use crate::utils::constants::POOL1_TXN;
use crate::utils::file::write_file;
use crate::utils::get_temp_dir_path;
use crate::utils::provision::provision_cloud_agent;
use crate::utils::test_logger::LibvcxDefaultLogger;

pub struct SetupEmpty;

pub struct SetupDefaults;

pub struct SetupMocks {}

#[derive(Clone)]
pub struct SetupProfile {
    pub institution_did: String,
    pub profile: Arc<dyn Profile>,
    pub teardown: Arc<dyn Fn() -> BoxFuture<'static, ()> + Send + Sync>,
    pub genesis_file_path: String,
}

pub struct SetupPoolDirectory {
    pub genesis_file_path: String,
}

pub fn reset_global_state() {
    warn!("reset_global_state >>");
    AgencyMockDecrypted::clear_mocks();
    PoolMocks::clear_mocks();
    DidMocks::clear_mocks();
    aries_vcx_disable_indy_mocks().unwrap();
    disable_indy_mocks_core().unwrap();
    reset_config_values_ariesvcx().unwrap();
    reset_config_values_ariesvcxcore().unwrap()
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
        enable_agency_mocks();
        aries_vcx_enable_indy_mocks().unwrap();
        enable_indy_mocks_core().unwrap();
        set_config_value(CONFIG_INSTITUTION_DID, DEFAULT_DID).unwrap();
        SetupMocks {}
    }
}

impl Drop for SetupMocks {
    fn drop(&mut self) {
        reset_global_state();
    }
}

#[cfg(feature = "migration")]
pub fn make_modular_profile(wallet_handle: WalletHandle, genesis_file_path: String) -> Arc<ModularLibsProfile> {
    let wallet = IndySdkWallet::new(wallet_handle);
    Arc::new(ModularLibsProfile::init(Arc::new(wallet), LedgerPoolConfig { genesis_file_path }).unwrap())
}

impl SetupProfile {
    pub async fn build_profile(genesis_file_path: String) -> SetupProfile {
        // In case of migration test setup, we are starting with vdrtools, then we migrate
        #[cfg(any(feature = "vdrtools", feature = "migration"))]
        return {
            info!("SetupProfile >> using indy profile");
            SetupProfile::build_profile_vdrtools(genesis_file_path).await
        };
        #[cfg(feature = "mixed_breed")]
        return {
            info!("SetupProfile >> using mixed breed profile");
            SetupProfile::build_profile_mixed_breed(genesis_file_path).await
        };

        #[cfg(feature = "modular_libs")]
        return {
            info!("SetupProfile >> using modular profile");
            SetupProfile::build_profile_modular(genesis_file_path).await
        };

        #[cfg(feature = "vdr_proxy_ledger")]
        return {
            info!("SetupProfile >> using vdr proxy profile");
            SetupProfile::build_profile_vdr_proxy_ledger(genesis_file_path).await
        };
    }

    #[cfg(feature = "vdrtools")]
    async fn build_profile_vdrtools(genesis_file_path: String) -> SetupProfile {
        let pool_name = Uuid::new_v4().to_string();
        create_pool_ledger_config(&pool_name, &genesis_file_path).unwrap();
        let pool_handle = indy_open_pool(&pool_name, None).await.unwrap();

        let (institution_did, wallet_handle) = setup_issuer_wallet().await;

        let profile = Arc::new(VdrtoolsProfile::init(wallet_handle, pool_handle.clone()));

        async fn indy_teardown(pool_handle: PoolHandle, pool_name: String) {
            indy_close_pool(pool_handle.clone()).await.unwrap();
            indy_delete_pool(&pool_name).await.unwrap();
        }

        SetupProfile {
            genesis_file_path,
            institution_did,
            profile,
            teardown: Arc::new(move || Box::pin(indy_teardown(pool_handle, pool_name.clone()))),
        }
    }

    #[cfg(feature = "modular_libs")]
    async fn build_profile_modular(genesis_file_path: String) -> SetupProfile {
        let (institution_did, wallet_handle) = setup_issuer_wallet().await;

        let wallet = IndySdkWallet::new(wallet_handle);

        let profile = Arc::new(
            ModularLibsProfile::init(
                Arc::new(wallet),
                LedgerPoolConfig {
                    genesis_file_path: genesis_file_path.clone(),
                },
            )
            .unwrap(),
        );

        // todo: this setup should be extracted out, is shared between profiles
        Arc::clone(&profile)
            .inject_anoncreds()
            .prover_create_link_secret(settings::DEFAULT_LINK_SECRET_ALIAS)
            .await
            .unwrap();

        async fn modular_teardown() {
            // nothing to do
        }

        SetupProfile {
            genesis_file_path,
            institution_did,
            profile,
            teardown: Arc::new(move || Box::pin(modular_teardown())),
        }
    }

    #[cfg(feature = "mixed_breed")]
    async fn build_profile_mixed_breed(genesis_file_path: String) -> SetupProfile {
        // todo: can remove?
        let pool_name = Uuid::new_v4().to_string();
        create_pool_ledger_config(&pool_name, &genesis_file_path).unwrap();
        let pool_handle = indy_open_pool(&pool_name, None).await.unwrap();

        let (institution_did, wallet_handle) = setup_issuer_wallet().await;

        let profile: Arc<dyn Profile> = Arc::new(MixedBreedProfile::new(wallet_handle, pool_handle.clone()));

        Arc::clone(&profile)
            .inject_anoncreds()
            .prover_create_link_secret(settings::DEFAULT_LINK_SECRET_ALIAS)
            .await
            .unwrap();

        async fn indy_teardown(pool_handle: PoolHandle, pool_name: String) {
            indy_close_pool(pool_handle.clone()).await.unwrap();
            indy_delete_pool(&pool_name).await.unwrap();
        }

        SetupProfile {
            genesis_file_path,
            institution_did,
            profile,
            teardown: Arc::new(move || Box::pin(indy_teardown(pool_handle, pool_name.clone()))),
        }
    }

    #[cfg(feature = "vdr_proxy_ledger")]
    async fn build_profile_vdr_proxy_ledger(genesis_file_path: String) -> SetupProfile {
        use std::env;

        use crate::core::profile::vdr_proxy_profile::VdrProxyProfile;
        use aries_vcx_core::VdrProxyClient;

        let client_url = env::var("VDR_PROXY_CLIENT_URL").unwrap_or_else(|_| "http://127.0.0.1:3030".to_string());
        let client = VdrProxyClient::new(&client_url).unwrap();

        let (institution_did, wallet_handle) = setup_issuer_wallet().await;

        let profile: Arc<dyn Profile> = Arc::new(VdrProxyProfile::init(wallet_handle, client).await.unwrap());

        async fn vdr_proxy_teardown() {
            // nothing to do
        }

        SetupProfile {
            genesis_file_path,
            institution_did,
            profile,
            teardown: Arc::new(move || Box::pin(vdr_proxy_teardown())),
        }
    }

    pub async fn run<F>(f: impl FnOnce(Self) -> F)
    where
        F: Future<Output = ()>,
    {
        init_test_logging();

        let genesis_file_path = get_temp_dir_path(POOL1_TXN).to_str().unwrap().to_string();
        create_testpool_genesis_txn_file(&genesis_file_path);

        warn!("genesis_file_path: {}", genesis_file_path);
        let init = Self::build_profile(genesis_file_path).await;

        let teardown = Arc::clone(&init.teardown);

        f(init).await;

        (teardown)().await;

        reset_global_state();
    }
}

impl SetupPoolDirectory {
    async fn init() -> SetupPoolDirectory {
        debug!("SetupPool init >> going to setup agency environment");
        init_test_logging();

        let genesis_file_path = get_temp_dir_path(POOL1_TXN).to_str().unwrap().to_string();
        create_testpool_genesis_txn_file(&genesis_file_path);

        debug!("SetupPool init >> completed");
        SetupPoolDirectory { genesis_file_path }
    }

    pub async fn run<F>(f: impl FnOnce(Self) -> F)
    where
        F: Future<Output = ()>,
    {
        let init = Self::init().await;

        f(init).await;

        // todo: delete the directory instead?
        // delete_test_pool(handle).await;

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
