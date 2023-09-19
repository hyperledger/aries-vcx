use std::sync::Arc;

use aries_vcx::core::profile::profile::Profile;

use crate::agent::agent_config::AgentConfig;

use crate::services::connection::ServiceConnections;
use crate::services::{
    credential_definition::ServiceCredentialDefinitions, holder::ServiceCredentialsHolder,
    issuer::ServiceCredentialsIssuer, prover::ServiceProver, revocation_registry::ServiceRevocationRegistries,
    schema::ServiceSchemas, verifier::ServiceVerifier,
};

#[derive(Clone)]
pub struct Agent {
    pub(super) profile: Arc<dyn Profile>,
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
    pub fn profile(&self) -> Arc<dyn Profile> {
        Arc::clone(&self.profile)
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
