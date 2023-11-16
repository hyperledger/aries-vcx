#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate indy_utils;

pub use indy_api_types as types;

#[macro_use]
mod utils;

#[macro_use]
mod controllers;
pub mod domain;
mod services;

use std::sync::Arc;

pub use domain::{
    anoncreds::{
        credential::{AttributeValues, Credential, CredentialValues},
        credential_definition::{
            CredentialDefinition, CredentialDefinitionCorrectnessProof, CredentialDefinitionData,
            CredentialDefinitionId, CredentialDefinitionPrivateKey, CredentialDefinitionV1,
            SignatureType,
        },
        credential_offer::CredentialOffer,
        credential_request::{CredentialRequest, CredentialRequestMetadata},
        master_secret::MasterSecret,
        revocation_registry::{RevocationRegistry, RevocationRegistryV1},
        revocation_registry_definition::{
            IssuanceType, RegistryType, RevocationRegistryConfig, RevocationRegistryDefinition,
            RevocationRegistryDefinitionPrivate, RevocationRegistryDefinitionV1,
            RevocationRegistryDefinitionValue, RevocationRegistryDefinitionValuePublicKeys,
            RevocationRegistryId, RevocationRegistryInfo,
        },
        revocation_registry_delta::{RevocationRegistryDelta, RevocationRegistryDeltaV1},
        schema::{AttributeNames, Schema, SchemaId, SchemaV1},
    },
    crypto::{
        did::{DidMethod, DidValue, MyDidInfo},
        key::KeyInfo,
        pack::JWE,
    },
};
pub use indy_api_types::{
    CommandHandle, IndyError, SearchHandle, WalletHandle, INVALID_COMMAND_HANDLE,
    INVALID_SEARCH_HANDLE, INVALID_WALLET_HANDLE,
};
pub use indy_wallet::WalletRecord;
use lazy_static::lazy_static;

use crate::{
    controllers::{CryptoController, DidController, NonSecretsController, WalletController},
    services::{CryptoService, WalletService},
};

// Global (lazy inited) instance of Locator
lazy_static! {
    static ref LOCATOR: Locator = Locator::new();
}

pub struct Locator {
    pub crypto_controller: CryptoController,
    pub did_controller: DidController,
    pub wallet_controller: WalletController,
    pub non_secret_controller: NonSecretsController,
}

impl Locator {
    pub fn instance() -> &'static Locator {
        &LOCATOR
    }

    fn new() -> Locator {
        info!("new >");

        let crypto_service = Arc::new(CryptoService::new());
        let wallet_service = Arc::new(WalletService::new());

        let crypto_controller =
            CryptoController::new(wallet_service.clone(), crypto_service.clone());

        let did_controller = DidController::new(wallet_service.clone(), crypto_service.clone());

        let wallet_controller = WalletController::new(wallet_service.clone(), crypto_service);
        let non_secret_controller = NonSecretsController::new(wallet_service);

        let res = Locator {
            crypto_controller,
            did_controller,
            wallet_controller,
            non_secret_controller,
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
