use agency_client::agency_client::AgencyClient;
use aries_vcx::aries_vcx_core::global::settings;
use aries_vcx::aries_vcx_core::indy::ledger::pool::test_utils::create_testpool_genesis_txn_file;
use aries_vcx::aries_vcx_core::indy::ledger::pool::{create_pool_ledger_config, indy_close_pool, indy_open_pool};
use aries_vcx::aries_vcx_core::{PoolHandle, WalletHandle};
use aries_vcx::global::settings::{
    set_config_value, CONFIG_GENESIS_PATH, CONFIG_INSTITUTION_DID, DEFAULT_DID, DEFAULT_GENESIS_PATH,
};
use aries_vcx::utils;
use std::future::Future;
use uuid::Uuid;

use aries_vcx::utils::devsetup::{init_test_logging, reset_global_state, setup_issuer_wallet_and_agency_client};
use aries_vcx::utils::get_temp_dir_path;

use crate::api_vcx::api_global::agency_client::{reset_main_agency_client, set_main_agency_client};
use crate::api_vcx::api_global::pool::{close_main_pool, reset_global_ledger_components, setup_ledger_components};
use crate::api_vcx::api_global::wallet::{close_main_wallet, setup_wallet};

pub struct SetupGlobalsWalletPoolAgency {
    pub agency_client: AgencyClient,
    pub institution_did: String,
    pub wallet_handle: WalletHandle,
    pub pool_handle: PoolHandle,
}

impl SetupGlobalsWalletPoolAgency {
    pub async fn init() -> SetupGlobalsWalletPoolAgency {
        reset_global_state();
        init_test_logging();
        set_config_value(CONFIG_INSTITUTION_DID, DEFAULT_DID).unwrap();
        let (institution_did, wallet_handle, agency_client) = setup_issuer_wallet_and_agency_client().await;
        let pool_name = Uuid::new_v4().to_string();
        let genesis_path = get_temp_dir_path(DEFAULT_GENESIS_PATH).to_str().unwrap().to_string();
        create_testpool_genesis_txn_file(&genesis_path);
        create_pool_ledger_config(&pool_name, &genesis_path).unwrap();
        let pool_handle = indy_open_pool(&pool_name, None).await.unwrap();
        SetupGlobalsWalletPoolAgency {
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

        let pool_handle = init.pool_handle.clone();
        setup_wallet(init.wallet_handle).unwrap();
        set_main_agency_client(init.agency_client.clone());
        setup_ledger_components(Some(pool_handle)).unwrap();

        f(init).await;

        close_main_wallet();
        reset_main_agency_client();
        indy_close_pool(pool_handle).await.unwrap();

        reset_global_state();
    }
}
