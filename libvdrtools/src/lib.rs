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

#[allow(unused_imports)]
#[macro_use]
extern crate indy_utils;

pub use indy_api_types as types;

#[macro_use]
mod utils;

#[macro_use]
mod controllers;
mod domain;
mod services;

use std::sync::Arc;

use lazy_static::lazy_static;

use crate::{
    controllers::{
        BlobStorageController, CacheController, ConfigController, CryptoController, DidController,
        IssuerController, LedgerController, NonSecretsController, PairwiseController,
        PoolController, ProverController, VerifierController, WalletController,
    },
    services::{
        BlobStorageService, CryptoService, IssuerService, LedgerService, PoolService,
        ProverService, VerifierService, WalletService,
    },
};

pub use controllers::CredentialDefinitionId;

pub use domain::{
    anoncreds::{
        credential::{Credential, CredentialValues},
        credential_definition::CredentialDefinition,
        credential_offer::CredentialOffer,
        credential_request::{CredentialRequest, CredentialRequestMetadata},
        revocation_registry_definition::{
            IssuanceType, RevocationRegistryConfig, RevocationRegistryDefinition,
            RevocationRegistryId,
        },
        revocation_state::RevocationStates,
        schema::{AttributeNames, Schema, SchemaId},
    },
    crypto::{
        did::{DidMethod, DidValue, MyDidInfo},
        key::KeyInfo,
        pack::JWE,
    },
    pool::PoolConfig,
};

pub use indy_api_types::{
    CommandHandle, PoolHandle, SearchHandle, WalletHandle, INVALID_COMMAND_HANDLE,
    INVALID_POOL_HANDLE, INVALID_SEARCH_HANDLE, INVALID_WALLET_HANDLE,
};

pub use services::AnoncredsHelpers;

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
        let pool_service = Arc::new(PoolService::new());
        let wallet_service = Arc::new(WalletService::new());

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
        let non_secret_controller = NonSecretsController::new(wallet_service.clone());

        let cache_controller = CacheController::new(
            crypto_service.clone(),
            ledger_service.clone(),
            pool_service.clone(),
            wallet_service.clone(),
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
