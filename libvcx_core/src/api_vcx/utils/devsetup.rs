use std::future::Future;

use aries_vcx::{
    aries_vcx_core::{
        ledger::indy::pool::test_utils::{create_testpool_genesis_txn_file, get_temp_file_path},
        wallet::indy::{
            wallet::{create_and_open_wallet, wallet_configure_issuer},
            WalletConfig,
        },
        WalletHandle,
    },
    global::settings::{self, DEFAULT_GENESIS_PATH},
    utils::devsetup::{init_test_logging, reset_global_state},
};

use crate::api_vcx::api_global::{
    pool::{close_main_pool, setup_ledger_components, LibvcxLedgerConfig},
    wallet::{close_main_wallet, setup_wallet},
};

async fn dev_setup_issuer_wallet_and_agency_client() -> (String, WalletHandle) {
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
    let wallet_handle = create_and_open_wallet(&config_wallet).await.unwrap();
    let config_issuer = wallet_configure_issuer(wallet_handle, enterprise_seed)
        .await
        .unwrap();

    (config_issuer.institution_did, wallet_handle)
}

pub struct SetupGlobalsWalletPoolAgency {
    pub institution_did: String,
    pub wallet_handle: WalletHandle,
}

impl SetupGlobalsWalletPoolAgency {
    pub async fn init() -> SetupGlobalsWalletPoolAgency {
        reset_global_state();
        init_test_logging();
        let (institution_did, wallet_handle) = dev_setup_issuer_wallet_and_agency_client().await;
        SetupGlobalsWalletPoolAgency {
            institution_did,
            wallet_handle,
        }
    }

    pub async fn run<F>(f: impl FnOnce(Self) -> F)
    where
        F: Future<Output = ()>,
    {
        let init = Self::init().await;

        let genesis_path = get_temp_file_path(DEFAULT_GENESIS_PATH)
            .to_str()
            .unwrap()
            .to_string();
        create_testpool_genesis_txn_file(&genesis_path);

        setup_wallet(init.wallet_handle).unwrap();
        let config = LibvcxLedgerConfig {
            genesis_path,
            pool_config: None,
            cache_config: None,
            exclude_nodes: None,
        };
        setup_ledger_components(&config).await.unwrap();

        f(init).await;

        close_main_wallet().await.ok();
        close_main_pool().await.ok();

        reset_global_state();
    }
}
