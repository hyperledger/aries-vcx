use std::sync::Arc;

use aries_vcx_core::{
    anoncreds::credx_anoncreds::IndyCredxAnonCreds,
    ledger::indy_vdr_ledger::{DefaultIndyLedgerRead, DefaultIndyLedgerWrite},
    wallet::base_wallet::BaseWallet,
};

use crate::{
    agent::agent_config::AgentConfig,
    services::{
        connection::ServiceConnections, credential_definition::ServiceCredentialDefinitions,
        did_exchange::ServiceDidExchange, holder::ServiceCredentialsHolder,
        issuer::ServiceCredentialsIssuer, out_of_band::ServiceOutOfBand, prover::ServiceProver,
        revocation_registry::ServiceRevocationRegistries, schema::ServiceSchemas,
        verifier::ServiceVerifier,
    },
};

#[derive(Clone)]
pub struct Agent<T> {
    pub(super) ledger_read: Arc<DefaultIndyLedgerRead>,
    pub(super) ledger_write: Arc<DefaultIndyLedgerWrite>,
    pub(super) anoncreds: IndyCredxAnonCreds,
    pub(super) wallet: Arc<T>,
    pub(super) config: AgentConfig,
    pub(super) connections: Arc<ServiceConnections<T>>,
    pub(super) schemas: Arc<ServiceSchemas<T>>,
    pub(super) cred_defs: Arc<ServiceCredentialDefinitions<T>>,
    pub(super) rev_regs: Arc<ServiceRevocationRegistries<T>>,
    pub(super) holder: Arc<ServiceCredentialsHolder<T>>,
    pub(super) issuer: Arc<ServiceCredentialsIssuer<T>>,
    pub(super) verifier: Arc<ServiceVerifier<T>>,
    pub(super) prover: Arc<ServiceProver<T>>,
    pub(super) out_of_band: Arc<ServiceOutOfBand<T>>,
    pub(super) did_exchange: Arc<ServiceDidExchange<T>>,
}

impl<T: BaseWallet> Agent<T> {
    pub fn ledger_read(&self) -> &DefaultIndyLedgerRead {
        &self.ledger_read
    }

    pub fn ledger_write(&self) -> &DefaultIndyLedgerWrite {
        &self.ledger_write
    }

    pub fn anoncreds(&self) -> &IndyCredxAnonCreds {
        &self.anoncreds
    }

    pub fn wallet(&self) -> &Arc<T> {
        &self.wallet
    }

    pub fn agent_config(&self) -> AgentConfig {
        self.config.clone()
    }

    pub fn issuer_did(&self) -> String {
        self.config.config_issuer.institution_did.clone()
    }

    pub fn connections(&self) -> Arc<ServiceConnections<T>> {
        self.connections.clone()
    }

    pub fn out_of_band(&self) -> Arc<ServiceOutOfBand<T>> {
        self.out_of_band.clone()
    }

    pub fn did_exchange(&self) -> Arc<ServiceDidExchange<T>> {
        self.did_exchange.clone()
    }

    pub fn schemas(&self) -> Arc<ServiceSchemas<T>> {
        self.schemas.clone()
    }

    pub fn cred_defs(&self) -> Arc<ServiceCredentialDefinitions<T>> {
        self.cred_defs.clone()
    }

    pub fn rev_regs(&self) -> Arc<ServiceRevocationRegistries<T>> {
        self.rev_regs.clone()
    }

    pub fn issuer(&self) -> Arc<ServiceCredentialsIssuer<T>> {
        self.issuer.clone()
    }

    pub fn holder(&self) -> Arc<ServiceCredentialsHolder<T>> {
        self.holder.clone()
    }

    pub fn verifier(&self) -> Arc<ServiceVerifier<T>> {
        self.verifier.clone()
    }

    pub fn prover(&self) -> Arc<ServiceProver<T>> {
        self.prover.clone()
    }

    pub fn public_did(&self) -> &str {
        self.did_exchange.public_did()
    }
}
