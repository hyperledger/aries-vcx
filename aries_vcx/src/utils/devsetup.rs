#![allow(clippy::unwrap_used)]

use std::fs;
use std::future::Future;
use std::sync::{Arc, Once};

use chrono::{DateTime, Duration, Utc};

use agency_client::agency_client::AgencyClient;
use agency_client::configuration::AgentProvisionConfig;
use agency_client::testing::mocking::{enable_agency_mocks, AgencyMockDecrypted};
use aries_vcx_core::global::settings::{
    disable_indy_mocks as disable_indy_mocks_core, enable_indy_mocks as enable_indy_mocks_core,
    reset_config_values_ariesvcxcore,
};
use aries_vcx_core::ledger::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite};
use aries_vcx_core::ledger::indy::pool::test_utils::{create_testpool_genesis_txn_file, get_temp_file_path};
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use aries_vcx_core::wallet::indy::did_mocks::DidMocks;
use aries_vcx_core::wallet::indy::wallet::{create_and_open_wallet, wallet_configure_issuer};
use aries_vcx_core::wallet::indy::{IndySdkWallet, WalletConfig};
use aries_vcx_core::WalletHandle;

use crate::core::profile::ledger::{build_ledger_components, VcxPoolConfig};
#[cfg(feature = "modular_libs")]
use crate::core::profile::modular_libs_profile::ModularLibsProfile;
use crate::core::profile::profile::Profile;
#[cfg(feature = "vdrtools")]
use crate::core::profile::vdrtools_profile::VdrtoolsProfile;
use crate::global::settings;
use crate::global::settings::{
    aries_vcx_disable_indy_mocks, aries_vcx_enable_indy_mocks, set_config_value, CONFIG_INSTITUTION_DID, DEFAULT_DID,
};
use crate::global::settings::{init_issuer_config, reset_config_values_ariesvcx};
use crate::utils::constants::{POOL1_TXN, TRUSTEE_SEED};
use crate::utils::file::write_file;
use crate::utils::provision::provision_cloud_agent;
use crate::utils::random::generate_random_seed;
use crate::utils::test_logger::LibvcxDefaultLogger;

#[macro_export]
macro_rules! assert_match {
    ($pattern:pat, $var:expr) => {
        assert!(match $var {
            $pattern => true,
            _ => false,
        })
    };
}

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

pub struct SetupEmpty;

pub struct SetupDefaults;

pub struct SetupMocks {}

pub const AGENCY_ENDPOINT: &str = "http://localhost:8080";
pub const AGENCY_DID: &str = "VsKV7grR1BUE29mG2Fm2kX";
pub const AGENCY_VERKEY: &str = "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR";

#[derive(Clone)]
pub struct SetupProfile {
    pub institution_did: String,
    pub profile: Arc<dyn Profile>,
    pub genesis_file_path: String,
}

pub struct SetupPoolDirectory {
    pub genesis_file_path: String,
}

pub fn reset_global_state() {
    warn!("reset_global_state >>");
    AgencyMockDecrypted::clear_mocks();
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

// todo: we move to libvcx?
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
    let wallet_handle = create_and_open_wallet(&config_wallet).await.unwrap();
    let config_issuer = wallet_configure_issuer(wallet_handle, enterprise_seed).await.unwrap();
    init_issuer_config(&config_issuer.institution_did).unwrap();
    let mut agency_client = AgencyClient::new();

    let wallet: Arc<dyn BaseWallet> = Arc::new(IndySdkWallet::new(wallet_handle));

    provision_cloud_agent(&mut agency_client, wallet, &config_provision_agent)
        .await
        .unwrap();

    (config_issuer.institution_did, wallet_handle, agency_client)
}

pub async fn setup_wallet_indy(key_seed: &str) -> (String, WalletHandle) {
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
    let wallet_handle = create_and_open_wallet(&config_wallet).await.unwrap();
    // todo: can we just extract thiss away? not always we end up using it (alice test agent)
    let config_issuer = wallet_configure_issuer(wallet_handle, key_seed).await.unwrap();
    // todo: can we remove this completely?
    init_issuer_config(&config_issuer.institution_did).unwrap();
    (config_issuer.institution_did, wallet_handle)
}

#[cfg(feature = "vdrtools")]
pub async fn dev_build_profile_vdrtools(genesis_file_path: String, wallet: Arc<IndySdkWallet>) -> Arc<dyn Profile> {
    let vcx_pool_config = VcxPoolConfig {
        genesis_file_path: genesis_file_path.clone(),
        indy_vdr_config: None,
        response_cache_config: None,
    };

    let (ledger_read, ledger_write) = build_ledger_components(wallet.clone(), vcx_pool_config).unwrap();
    let anoncreds_ledger_read: Arc<dyn AnoncredsLedgerRead> = ledger_read.clone();
    let anoncreds_ledger_write: Arc<dyn AnoncredsLedgerWrite> = ledger_write.clone();
    let indy_ledger_read: Arc<dyn IndyLedgerRead> = ledger_read.clone();
    let indy_ledger_write: Arc<dyn IndyLedgerWrite> = ledger_write.clone();
    Arc::new(VdrtoolsProfile::init(
        wallet.clone(),
        anoncreds_ledger_read,
        anoncreds_ledger_write,
        indy_ledger_read,
        indy_ledger_write,
    ))
}

#[cfg(feature = "modular_libs")]
pub fn dev_build_profile_modular(genesis_file_path: String, wallet: Arc<IndySdkWallet>) -> Arc<dyn Profile> {
    let vcx_pool_config = VcxPoolConfig {
        genesis_file_path: genesis_file_path.clone(),
        indy_vdr_config: None,
        response_cache_config: None,
    };
    Arc::new(ModularLibsProfile::init(wallet, vcx_pool_config).unwrap())
}

#[cfg(feature = "vdr_proxy_ledger")]
pub async fn dev_build_profile_vdr_proxy_ledger(wallet: Arc<IndySdkWallet>) -> Arc<dyn Profile> {
    use crate::core::profile::vdr_proxy_profile::VdrProxyProfile;
    use aries_vcx_core::VdrProxyClient;
    use std::env;

    let client_url = env::var("VDR_PROXY_CLIENT_URL").unwrap_or_else(|_| "http://127.0.0.1:3030".to_string());
    let client = VdrProxyClient::new(&client_url).unwrap();

    Arc::new(VdrProxyProfile::init(wallet, client).await.unwrap())
}

async fn build_featured_profile(genesis_file_path: String, wallet: Arc<IndySdkWallet>) -> Arc<dyn Profile> {
    // In case of migration test setup, we are starting with vdrtools, then we migrate
    #[cfg(any(feature = "vdrtools", feature = "migration"))]
    return {
        info!("SetupProfile >> using indy profile");
        dev_build_profile_vdrtools(genesis_file_path, wallet).await
    };
    #[cfg(feature = "modular_libs")]
    return {
        info!("SetupProfile >> using modular profile");
        dev_build_profile_modular(genesis_file_path, wallet)
    };
    #[cfg(feature = "vdr_proxy_ledger")]
    return {
        info!("SetupProfile >> using vdr proxy profile");
        dev_build_profile_vdr_proxy_ledger(wallet).await
    };
}

impl SetupProfile {
    async fn build(genesis_file_path: String, public_did: &str, wallet_handle: WalletHandle) -> SetupProfile {
        let wallet = Arc::new(IndySdkWallet::new(wallet_handle));
        let profile = build_featured_profile(genesis_file_path.clone(), wallet).await;
        profile
            .inject_anoncreds()
            .prover_create_link_secret(settings::DEFAULT_LINK_SECRET_ALIAS)
            .await
            .unwrap();
        SetupProfile {
            institution_did: public_did.to_string(),
            profile,
            genesis_file_path,
        }
    }

    pub async fn build_with_trustee_did(genesis_file_path: String) -> SetupProfile {
        let (public_did, wh) = setup_wallet_indy(TRUSTEE_SEED).await;
        Self::build(genesis_file_path, &public_did, wh).await
    }

    pub async fn build_with_random_did(genesis_file_path: String) -> SetupProfile {
        let (public_did, wh) = setup_wallet_indy(&generate_random_seed()).await;
        Self::build(genesis_file_path, &public_did, wh).await
    }

    pub async fn run<F>(f: impl FnOnce(Self) -> F)
    where
        F: Future<Output = ()>,
    {
        init_test_logging();

        let genesis_file_path = get_temp_file_path(POOL1_TXN).to_str().unwrap().to_string();
        create_testpool_genesis_txn_file(&genesis_file_path);

        warn!("genesis_file_path: {}", genesis_file_path);
        let setup = Self::build_with_trustee_did(genesis_file_path).await;
        // todo: this setup should be extracted out, is shared between profiles

        f(setup).await;
        reset_global_state();
    }
}

impl SetupPoolDirectory {
    async fn init() -> SetupPoolDirectory {
        debug!("SetupPool init >> going to setup agency environment");
        init_test_logging();

        let genesis_file_path = get_temp_file_path(POOL1_TXN).to_str().unwrap().to_string();
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
