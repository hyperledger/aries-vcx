#![cfg_attr(feature = "fatal_warnings", deny(warnings))]
#[macro_use]
extern crate derivative;

#[macro_use]
extern crate log;

extern crate serde;

extern crate variant_count;

extern crate num_cpus;

#[macro_use]
extern crate num_derive;

extern crate num_traits;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate indy_utils;

#[macro_use]
mod utils;

#[macro_use]
mod controllers;
mod domain;
mod services;

pub mod api;

use std::sync::Arc;

use lazy_static::lazy_static;

use crate::{
    controllers::{
        BlobStorageController, CacheController, ConfigController, CryptoController, DidController,
        IssuerController, LedgerController, MetricsController, NonSecretsController,
        PairwiseController, PoolController, ProverController, VerifierController, WalletController,
        VDRController,
    },
    services::{
        BlobStorageService, CryptoService, IssuerService, LedgerService, MetricsService,
        CommandMetric, PoolService, ProverService, VerifierService, WalletService,
    },
};

use indy_api_types::errors::IndyResult;
use std::future::Future;
use std::time::{SystemTime, UNIX_EPOCH};
use std::cmp;

fn get_cur_time() -> u128 {
    let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time has gone backwards");
    since_epoch.as_millis()
}

#[derive(Clone)]
pub struct InstrumentedThreadPool {
    executor: futures::executor::ThreadPool,
    metrics_service: Arc<MetricsService>,
}

impl InstrumentedThreadPool {
    pub fn spawn_ok_instrumented<T, FutIndyRes, FnCb>(&self, idx: CommandMetric, action: FutIndyRes, cb: FnCb)
        where
            FutIndyRes: Future<Output = IndyResult<T>> + Send + 'static,
            FnCb: Fn(IndyResult<T>) + Sync + Send + 'static,
            T: Send + 'static
    {
        let requested_time = get_cur_time();
        let metrics_service = self.metrics_service.clone();
        self.executor.spawn_ok(async move {
            let start_time = get_cur_time();
            let res = action.await;
            let executed_time = get_cur_time();
            cb(res);
            let cb_finished_time = get_cur_time();
            metrics_service.cmd_left_queue(idx, start_time - requested_time).await;
            metrics_service.cmd_executed(idx, executed_time - start_time).await;
            metrics_service.cmd_callback(idx, cb_finished_time - executed_time).await;
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
    pub vdr_controller: VDRController,
    pub executor: InstrumentedThreadPool,
}


impl Locator {
    pub fn instance() -> &'static Locator {
        &LOCATOR
    }

    fn new() -> Locator {
        info!("new >");

        std::panic::set_hook(Box::new(|pi| {
            error!("Custom panic hook");
            error!("Custom panic hook: {:?}", pi);
            let bt = backtrace::Backtrace::new();
            error!("Custom panic hook: {:?}", bt);
        }));

        let issuer_service = Arc::new(IssuerService::new());
        let prover_service = Arc::new(ProverService::new());
        let verifier_service = Arc::new(VerifierService::new());
        let blob_storage_service = Arc::new(BlobStorageService::new());
        let crypto_service = Arc::new(CryptoService::new());
        let ledger_service = Arc::new(LedgerService::new());
        let metrics_service = Arc::new(MetricsService::new());
        let pool_service = Arc::new(PoolService::new());
        let wallet_service = Arc::new(WalletService::new());

        // TODO: Make it work with lower number of threads (VE-2668)
        let num_threads = cmp::max(8, num_cpus::get());
        let executor = InstrumentedThreadPool {
            executor: futures::executor::ThreadPool::builder().pool_size(num_threads).create().unwrap(),
            metrics_service: metrics_service.clone(),
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
        let metrics_controller = MetricsController::new(wallet_service.clone(), metrics_service.clone());
        let non_secret_controller = NonSecretsController::new(wallet_service.clone());

        let cache_controller = CacheController::new(
            crypto_service.clone(),
            ledger_service.clone(),
            pool_service.clone(),
            wallet_service.clone(),
        );

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
            vdr_controller,
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
