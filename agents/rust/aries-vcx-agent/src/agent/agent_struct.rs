use std::sync::Arc;

use aries_vcx::{
    agency_client::agency_client::AgencyClient, core::profile::profile::Profile,
    plugins::wallet::agency_client_wallet::ToBaseAgencyClientWallet,
};

use crate::{
    agent::agent_config::AgentConfig,
    error::*,
    services::{
        connection::ServiceConnections, credential_definition::ServiceCredentialDefinitions,
        holder::ServiceCredentialsHolder, issuer::ServiceCredentialsIssuer,
        mediated_connection::ServiceMediatedConnections, prover::ServiceProver,
        revocation_registry::ServiceRevocationRegistries, schema::ServiceSchemas, verifier::ServiceVerifier,
    },
};

#[derive(Clone)]
pub struct Agent {
    pub(super) profile: Arc<dyn Profile>,
    pub(super) config: AgentConfig,
    pub(super) connections: Arc<ServiceConnections>,
    pub(super) mediated_connections: Option<Arc<ServiceMediatedConnections>>,
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

    pub fn agency_client(&self) -> AgentResult<AgencyClient> {
        if let Some(config_agency_client) = &self.config.config_agency_client {
            let wallet = self.profile.inject_wallet();
            AgencyClient::new()
                .configure(wallet.to_base_agency_client_wallet(), config_agency_client)
                .map_err(|err| {
                    AgentError::from_msg(
                        AgentErrorKind::GenericAriesVcxError,
                        &format!("Failed to configure agency client: {}", err),
                    )
                })
        } else {
            Err(AgentError::from_kind(
                AgentErrorKind::MediatedConnectionServiceUnavailable,
            ))
        }
    }

    pub fn connections(&self) -> Arc<ServiceConnections> {
        self.connections.clone()
    }

    pub fn mediated_connections(&self) -> AgentResult<Arc<ServiceMediatedConnections>> {
        self.mediated_connections
            .clone()
            .ok_or_else(|| AgentError::from_kind(AgentErrorKind::MediatedConnectionServiceUnavailable))
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
