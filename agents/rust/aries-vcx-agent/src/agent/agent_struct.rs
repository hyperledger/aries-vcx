use std::sync::Arc;

use aries_vcx_core::{anoncreds::credx_anoncreds::IndyCredxAnonCreds, wallet::indy::IndySdkWallet};
use test_utils::devsetup::{DefaultIndyLedgerRead, DefaultIndyLedgerWrite};

use crate::{
    agent::agent_config::AgentConfig,
    services::{
        connection::ServiceConnections, credential_definition::ServiceCredentialDefinitions,
        holder::ServiceCredentialsHolder, issuer::ServiceCredentialsIssuer, prover::ServiceProver,
        revocation_registry::ServiceRevocationRegistries, schema::ServiceSchemas,
        verifier::ServiceVerifier,
    },
};

#[derive(Clone)]
pub struct Agent {
    pub(super) ledger_read: Arc<DefaultIndyLedgerRead>,
    pub(super) ledger_write: Arc<DefaultIndyLedgerWrite>,
    pub(super) anoncreds: IndyCredxAnonCreds,
    pub(super) wallet: Arc<IndySdkWallet>,
    pub(super) config: AgentConfig,
    pub(super) connections: Arc<ServiceConnections>,
    pub(super) schemas: Arc<ServiceSchemas>,
    pub(super) cred_defs: Arc<ServiceCredentialDefinitions>,
    pub(super) rev_regs: Arc<ServiceRevocationRegistries>,
    pub(super) holder: Arc<ServiceCredentialsHolder>,
    pub(super) issuer: Arc<ServiceCredentialsIssuer>,
    pub(super) verifier: Arc<ServiceVerifier>,
    pub(super) prover: Arc<ServiceProver>,
}

impl Agent {
    pub fn ledger_read(&self) -> &DefaultIndyLedgerRead {
        &self.ledger_read
    }

    pub fn ledger_write(&self) -> &DefaultIndyLedgerWrite {
        &self.ledger_write
    }

    pub fn anoncreds(&self) -> &IndyCredxAnonCreds {
        &self.anoncreds
    }

    pub fn wallet(&self) -> &IndySdkWallet {
        &self.wallet
    }

    pub fn agent_config(&self) -> AgentConfig {
        self.config.clone()
    }

    pub fn issuer_did(&self) -> String {
        self.config.config_issuer.institution_did.clone()
    }

    pub fn connections(&self) -> Arc<ServiceConnections> {
        self.connections.clone()
    }

    pub fn schemas(&self) -> Arc<ServiceSchemas> {
        self.schemas.clone()
    }

    pub fn cred_defs(&self) -> Arc<ServiceCredentialDefinitions> {
        self.cred_defs.clone()
    }

    pub fn rev_regs(&self) -> Arc<ServiceRevocationRegistries> {
        self.rev_regs.clone()
    }

    pub fn issuer(&self) -> Arc<ServiceCredentialsIssuer> {
        self.issuer.clone()
    }

    pub fn holder(&self) -> Arc<ServiceCredentialsHolder> {
        self.holder.clone()
    }

    pub fn verifier(&self) -> Arc<ServiceVerifier> {
        self.verifier.clone()
    }

    pub fn prover(&self) -> Arc<ServiceProver> {
        self.prover.clone()
    }
}
