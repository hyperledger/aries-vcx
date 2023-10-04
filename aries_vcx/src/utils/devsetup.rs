#![allow(clippy::unwrap_used)]

use std::{
    fs,
    future::Future,
    sync::{Arc, Once},
};

use agency_client::testing::mocking::{enable_agency_mocks, AgencyMockDecrypted};
use aries_vcx_core::{
    ledger::indy::pool::test_utils::{create_testpool_genesis_txn_file, get_temp_file_path},
    wallet::indy::{
        did_mocks::DidMocks,
        wallet::{create_and_open_wallet, create_and_store_my_did},
        IndySdkWallet, WalletConfig,
    },
    WalletHandle,
};
use chrono::{DateTime, Duration, Utc};

#[cfg(feature = "modular_libs")]
use crate::core::profile::modular_libs_profile::ModularLibsProfile;
#[cfg(feature = "vdr_proxy_ledger")]
use crate::core::profile::vdr_proxy_profile::VdrProxyProfile;
#[cfg(feature = "vdrtools")]
use crate::core::profile::vdrtools_profile::VdrtoolsProfile;
use crate::{
    core::profile::{ledger::VcxPoolConfig, Profile},
    global::settings,
    utils::{constants::POOL1_TXN, file::write_file, test_logger::LibvcxDefaultLogger},
};

lazy_static! {
    static ref TEST_LOGGING_INIT: Once = Once::new();
}

pub fn init_test_logging() {
    TEST_LOGGING_INIT.call_once(|| {
        LibvcxDefaultLogger::init_testing_logger();
    })
}

pub struct SetupEmpty;

pub struct SetupDefaults;

pub struct SetupMocks;

pub const AGENCY_ENDPOINT: &str = "http://localhost:8080";
pub const AGENCY_DID: &str = "VsKV7grR1BUE29mG2Fm2kX";
pub const AGENCY_VERKEY: &str = "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR";

#[derive(Clone)]
pub struct SetupProfile<P: Profile> {
    pub institution_did: String,
    pub profile: P,
    pub genesis_file_path: String,
}

pub struct SetupPoolDirectory {
    pub genesis_file_path: String,
}

pub fn reset_global_state() {
    warn!("reset_global_state >>");
    AgencyMockDecrypted::clear_mocks();
    DidMocks::clear_mocks();
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
        SetupMocks {}
    }
}

impl Drop for SetupMocks {
    fn drop(&mut self) {
        reset_global_state();
    }
}

pub async fn dev_setup_wallet_indy(key_seed: &str) -> (String, WalletHandle) {
    info!("dev_setup_wallet_indy >>");
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
    let wallet_handle = create_and_open_wallet(&config_wallet).await.unwrap();
    // todo: can we just extract this away? not always we end up using it (alice test agent)
    let (did, _vk) = create_and_store_my_did(wallet_handle, Some(key_seed), None)
        .await
        .unwrap();

    (did, wallet_handle)
}

#[cfg(feature = "vdrtools")]
pub fn dev_build_profile_vdrtools(
    genesis_file_path: String,
    wallet: Arc<IndySdkWallet>,
) -> VdrtoolsProfile {
    info!("dev_build_profile_vdrtools >>");
    let vcx_pool_config = VcxPoolConfig {
        genesis_file_path,
        indy_vdr_config: None,
        response_cache_config: None,
    };
    VdrtoolsProfile::init(wallet, vcx_pool_config).unwrap()
}

#[cfg(feature = "modular_libs")]
pub fn dev_build_profile_modular(
    genesis_file_path: String,
    wallet: Arc<IndySdkWallet>,
) -> ModularLibsProfile {
    info!("dev_build_profile_modular >>");
    let vcx_pool_config = VcxPoolConfig {
        genesis_file_path,
        indy_vdr_config: None,
        response_cache_config: None,
    };
    ModularLibsProfile::init(wallet, vcx_pool_config).unwrap()
}

#[cfg(feature = "vdr_proxy_ledger")]
pub async fn dev_build_profile_vdr_proxy_ledger(wallet: Arc<IndySdkWallet>) -> VdrProxyProfile {
    use std::env;

    use aries_vcx_core::VdrProxyClient;

    info!("dev_build_profile_vdr_proxy_ledger >>");

    let client_url =
        env::var("VDR_PROXY_CLIENT_URL").unwrap_or_else(|_| "http://127.0.0.1:3030".to_string());
    let client = VdrProxyClient::new(&client_url).unwrap();

    VdrProxyProfile::init(wallet, client).await.unwrap()
}

#[allow(unreachable_code)]
#[allow(unused_variables)]
pub async fn dev_build_featured_profile(
    genesis_file_path: String,
    wallet: Arc<IndySdkWallet>,
) -> impl Profile {
    // In case of migration test setup, we are starting with vdrtools, then we migrate
    #[cfg(all(feature = "modular_libs", not(feature = "migration")))]
    return {
        info!("SetupProfile >> using modular profile");
        dev_build_profile_modular(genesis_file_path, wallet)
    };
    #[cfg(all(feature = "vdr_proxy_ledger", not(feature = "migration")))]
    return {
        info!("SetupProfile >> using vdr proxy profile");
        dev_build_profile_vdr_proxy_ledger(wallet).await
    };
    #[cfg(any(
        all(
            feature = "vdrtools",
            not(feature = "vdr_proxy_ledger"),
            not(feature = "modular_libs")
        ),
        feature = "migration"
    ))]
    return {
        info!("SetupProfile >> using indy profile");
        dev_build_profile_vdrtools(genesis_file_path, wallet)
    };
}

#[macro_export]
macro_rules! run_setup {
    ($func:expr) => {{
        use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
        use $crate::core::profile::Profile;

        $crate::utils::devsetup::init_test_logging();

        let genesis_file_path = aries_vcx_core::ledger::indy::pool::test_utils::get_temp_file_path(
            $crate::utils::constants::POOL1_TXN,
        )
        .to_str()
        .unwrap()
        .to_string();
        aries_vcx_core::ledger::indy::pool::test_utils::create_testpool_genesis_txn_file(
            &genesis_file_path,
        );

        let (public_did, wallet_handle) =
            $crate::utils::devsetup::dev_setup_wallet_indy($crate::utils::constants::TRUSTEE_SEED)
                .await;
        let wallet = std::sync::Arc::new(aries_vcx_core::wallet::indy::IndySdkWallet::new(
            wallet_handle,
        ));
        let profile =
            $crate::utils::devsetup::dev_build_featured_profile(genesis_file_path.clone(), wallet)
                .await;
        profile
            .anoncreds()
            .prover_create_link_secret(aries_vcx_core::global::settings::DEFAULT_LINK_SECRET_ALIAS)
            .await
            .unwrap();

        $crate::utils::devsetup::SetupProfile::new(
            public_did.to_string(),
            profile,
            genesis_file_path,
        )
        .await
        .run($func)
    }};
}

impl<P> SetupProfile<P>
where
    P: Profile,
{
    pub async fn new(institution_did: String, profile: P, genesis_file_path: String) -> Self {
        debug!("genesis_file_path: {}", genesis_file_path);
        SetupProfile {
            institution_did,
            profile,
            genesis_file_path,
        }
    }

    pub async fn run<F>(self, f: impl FnOnce(Self) -> F)
    where
        F: Future<Output = ()>,
    {
        f(self).await;
        reset_global_state();
    }
}

impl SetupPoolDirectory {
    async fn init() -> SetupPoolDirectory {
        debug!("SetupPoolDirectory init >> going to setup agency environment");
        init_test_logging();

        let genesis_file_path = get_temp_file_path(POOL1_TXN).to_str().unwrap().to_string();
        create_testpool_genesis_txn_file(&genesis_file_path);

        debug!("SetupPoolDirectory init >> completed");
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
