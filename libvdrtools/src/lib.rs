#![cfg_attr(feature = "fatal_warnings", deny(warnings))]

#[macro_use]
extern crate log;

extern crate variant_count;

#[macro_use]
extern crate num_derive;

extern crate num_traits;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

// #[cfg(feature = "ffi_api")]
// #[macro_use]
extern crate indy_utils;

pub use indy_api_types as types;

#[macro_use]
mod utils;

#[macro_use]
mod controllers;
mod domain;
mod services;

#[cfg(feature = "ffi_api")]
pub mod api;

use std::sync::Arc;

use lazy_static::lazy_static;

#[cfg(feature = "ffi_api")]
use crate::services::CommandMetric;

#[cfg(feature = "ffi_api")]
use crate::controllers::VDRController;

use crate::{
    controllers::{
        BlobStorageController, CacheController, ConfigController, CryptoController, DidController,
        IssuerController, LedgerController, MetricsController, NonSecretsController,
        PairwiseController, PoolController, ProverController, VerifierController,
        WalletController,
    },

    services::{
        BlobStorageService,
        CryptoService, IssuerService, LedgerService,
        MetricsService, PoolService, ProverService, VerifierService, WalletService,
    },
};

#[cfg(feature = "ffi_api")]
use indy_api_types::errors::IndyResult;

#[cfg(feature = "ffi_api")]
use std::{
    cmp,
    future::Future,
    time::{SystemTime, UNIX_EPOCH},
};

pub use controllers::CredentialDefinitionId;

pub use domain::{
    anoncreds::{
        revocation_registry_definition::{
            RevocationRegistryId,
            RevocationRegistryDefinition,
            RevocationRegistryConfig,
            IssuanceType,
        },
        revocation_state::RevocationStates,
        credential::{CredentialValues, Credential},
        credential_request::{CredentialRequest, CredentialRequestMetadata},
        credential_definition::CredentialDefinition,
        credential_offer::CredentialOffer,
        schema::AttributeNames,
        schema::{Schema, SchemaId},
    },
    crypto::{
        did::{
            DidMethod, DidValue, MyDidInfo,
        },
        key::KeyInfo,
        pack::JWE,
    },
    pool::PoolConfig,
};

pub use indy_api_types::{
    WalletHandle, INVALID_WALLET_HANDLE,
    SearchHandle, INVALID_SEARCH_HANDLE,
    PoolHandle, INVALID_POOL_HANDLE,
    CommandHandle, INVALID_COMMAND_HANDLE,
};

pub use services::AnoncredsHelpers;

#[cfg(feature = "ffi_api")]
fn get_cur_time() -> u128 {
    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time has gone backwards");
    since_epoch.as_millis()
}

#[cfg(feature = "ffi_api")]
#[derive(Clone)]
pub struct InstrumentedThreadPool {
    executor: futures::executor::ThreadPool,
    metrics_service: Arc<MetricsService>,
}

#[cfg(feature = "ffi_api")]
impl InstrumentedThreadPool {
    pub fn spawn_ok_instrumented<T, FutIndyRes, FnCb>(
        &self,
        idx: CommandMetric,
        action: FutIndyRes,
        cb: FnCb,
    ) where
        FutIndyRes: Future<Output = IndyResult<T>> + Send + 'static,
        FnCb: Fn(IndyResult<T>) + Sync + Send + 'static,
        T: Send + 'static,
    {
        let requested_time = get_cur_time();
        let metrics_service = self.metrics_service.clone();
        self.executor.spawn_ok(async move {
            let start_time = get_cur_time();
            let res = action.await;
            let executed_time = get_cur_time();
            cb(res);
            let cb_finished_time = get_cur_time();
            metrics_service
                .cmd_left_queue(idx, start_time - requested_time)
                .await;
            metrics_service
                .cmd_executed(idx, executed_time - start_time)
                .await;
            metrics_service
                .cmd_callback(idx, cb_finished_time - executed_time)
                .await;
        })
    }
}

// Global (lazy inited) instance of Locator
lazy_static! {
    static ref LOCATOR: Locator = Locator::new();
}

pub struct Locator {
    pub issuer_controller: IssuerController,
    pub prover_controller: ProverController,
    pub verifier_controller: VerifierController,
    pub crypto_controller: CryptoController,
    pub config_controller: ConfigController,
    pub ledger_controller: LedgerController,
    pub pool_controller: PoolController,
    pub did_controller: DidController,
    pub wallet_controller: WalletController,
    pub pairwise_controller: PairwiseController,
    pub blob_storage_controller: BlobStorageController,
    pub non_secret_controller: NonSecretsController,
    pub cache_controller: CacheController,
    pub metrics_controller: MetricsController,

    #[cfg(feature = "ffi_api")]
    pub vdr_controller: VDRController,

    #[cfg(feature = "ffi_api")]
    pub executor: InstrumentedThreadPool,
}

impl Locator {
    pub fn instance() -> &'static Locator {
        &LOCATOR
    }

    fn new() -> Locator {
        info!("new >");

        let issuer_service = Arc::new(IssuerService::new());
        let prover_service = Arc::new(ProverService::new());
        let verifier_service = Arc::new(VerifierService::new());
        let blob_storage_service = Arc::new(BlobStorageService::new());
        let crypto_service = Arc::new(CryptoService::new());
        let ledger_service = Arc::new(LedgerService::new());
        let metrics_service = Arc::new(MetricsService::new());
        let pool_service = Arc::new(PoolService::new());
        let wallet_service = Arc::new(WalletService::new());

        #[cfg(feature = "ffi_api")]
        let executor = {
            // TODO: Make it work with lower number of threads (VE-2668)
            let num_threads = cmp::max(8, num_cpus::get());

            InstrumentedThreadPool {
                executor: futures::executor::ThreadPool::builder()
                    .pool_size(num_threads)
                    .create()
                    .unwrap(),
                metrics_service: metrics_service.clone(),
            }
        };

        let issuer_controller = IssuerController::new(
            issuer_service,
            blob_storage_service.clone(),
            wallet_service.clone(),
            crypto_service.clone(),
        );

        let prover_controller = ProverController::new(
            prover_service,
            wallet_service.clone(),
            crypto_service.clone(),
            blob_storage_service.clone(),
        );

        let verifier_controller = VerifierController::new(verifier_service);

        let crypto_controller =
            CryptoController::new(wallet_service.clone(), crypto_service.clone());

        let config_controller = ConfigController::new();

        let ledger_controller = LedgerController::new(
            pool_service.clone(),
            crypto_service.clone(),
            wallet_service.clone(),
            ledger_service.clone(),
        );

        let pool_controller = PoolController::new(pool_service.clone());

        let did_controller = DidController::new(
            wallet_service.clone(),
            crypto_service.clone(),
            ledger_service.clone(),
            pool_service.clone(),
        );

        let wallet_controller =
            WalletController::new(wallet_service.clone(), crypto_service.clone());

        let pairwise_controller = PairwiseController::new(wallet_service.clone());
        let blob_storage_controller = BlobStorageController::new(blob_storage_service.clone());
        let metrics_controller =
            MetricsController::new(wallet_service.clone(), metrics_service.clone());
        let non_secret_controller = NonSecretsController::new(wallet_service.clone());

        let cache_controller = CacheController::new(
            crypto_service.clone(),
            ledger_service.clone(),
            pool_service.clone(),
            wallet_service.clone(),
        );

        #[cfg(feature = "ffi_api")]
        let vdr_controller = VDRController::new(
            wallet_service.clone(),
            ledger_service.clone(),
            pool_service.clone(),
            crypto_service.clone(),
        );

        let res = Locator {
            issuer_controller,
            prover_controller,
            verifier_controller,
            crypto_controller,
            config_controller,
            ledger_controller,
            pool_controller,
            did_controller,
            wallet_controller,
            pairwise_controller,
            blob_storage_controller,
            non_secret_controller,
            cache_controller,
            metrics_controller,

            #[cfg(feature = "ffi_api")]
            vdr_controller,

            #[cfg(feature = "ffi_api")]
            executor,
        };

        info!("new <");
        res
    }
}

impl Drop for Locator {
    fn drop(&mut self) {
        info!(target: "Locator", "drop <>");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locator_new_works() {
        let _locator = Locator::new();
        assert!(true);
    }

    #[test]
    fn locator_drop_works() {
        {
            let _locator = Locator::new();
        }

        assert!(true);
    }

    #[test]
    fn locator_get_instance_works() {
        let locator = Locator::instance();
        let locator2 = Locator::instance();
        assert!(std::ptr::eq(locator, locator2));
    }
}
