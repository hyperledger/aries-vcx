use std::sync::Arc;

use crate::error::*;
use crate::storage::in_memory::ObjectCache;
use aries_vcx::agency_client::agency_client::AgencyClient;
use aries_vcx::agency_client::configuration::AgencyClientConfig;
use aries_vcx::handlers::proof_presentation::verifier::Verifier;
use aries_vcx::indy::proofs::proof_request::PresentationRequestData;
use aries_vcx::messages::proof_presentation::presentation_proposal::PresentationProposal;
use aries_vcx::messages::status::Status;
use aries_vcx::protocols::proof_presentation::verifier::state_machine::VerifierState;
use aries_vcx::vdrtools_sys::{PoolHandle, WalletHandle};

use super::connection::ServiceConnections;

#[derive(Clone)]
struct VerifierWrapper {
    verifier: Verifier,
    connection_id: String,
}

impl VerifierWrapper {
    pub fn new(verifier: Verifier, connection_id: &str) -> Self {
        Self {
            verifier,
            connection_id: connection_id.to_string(),
        }
    }
}

pub struct ServiceVerifier {
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    config_agency_client: AgencyClientConfig,
    verifiers: ObjectCache<VerifierWrapper>,
    service_connections: Arc<ServiceConnections>,
}

impl ServiceVerifier {
    pub fn new(wallet_handle: WalletHandle, pool_handle: PoolHandle, config_agency_client: AgencyClientConfig, service_connections: Arc<ServiceConnections>) -> Self {
        Self {
            wallet_handle,
            pool_handle,
            config_agency_client,
            service_connections,
            verifiers: ObjectCache::new("verifiers"),
        }
    }

    fn agency_client(&self) -> AgentResult<AgencyClient> {
        AgencyClient::new()
            .configure(self.wallet_handle, &self.config_agency_client)
            .map_err(|err| {
                AgentError::from_msg(
                    AgentErrorKind::GenericAriesVcxError,
                    &format!("Failed to configure agency client: {}", err),
                )
            })
    }

    pub async fn send_proof_request(&self, connection_id: &str, request: PresentationRequestData, proposal: Option<PresentationProposal>) -> AgentResult<String> {
        let connection = self.service_connections.get_by_id(connection_id)?;
        let mut verifier = if let Some(proposal) = proposal {
            Verifier::create_from_proposal("", &proposal)?
        } else {
            Verifier::create_from_request("".to_string(), &request)?
        };
        verifier
            .send_presentation_request(
                connection
                    .send_message_closure(self.wallet_handle)
                    .await?,
            )
            .await?;
        self.verifiers.add(&verifier.get_thread_id()?, VerifierWrapper::new(verifier, connection_id))
    }

    pub fn verify_presentation(&self, id: &str) -> AgentResult<Status> {
        let VerifierWrapper { verifier, .. } = self.verifiers.get_cloned(id)?;
        Ok(Status::from_u32(verifier.get_presentation_status()))
    }

    pub async fn update_state(&self, id: &str) -> AgentResult<VerifierState> {
        let VerifierWrapper { mut verifier, connection_id } = self.verifiers.get_cloned(id)?;
        let connection = self.service_connections.get_by_id(&connection_id)?;
        let state = verifier
            .update_state(
                self.wallet_handle,
                self.pool_handle,
                &self.agency_client()?,
                &connection
            )
            .await?;
        self.verifiers.add(&verifier.get_thread_id()?, VerifierWrapper::new(verifier, &connection_id))?;
        Ok(state)
    }

    pub fn get_state(&self, id: &str) -> AgentResult<VerifierState> {
        let VerifierWrapper { verifier, .. } = self.verifiers.get_cloned(id)?;
        Ok(verifier.get_state())
    }

    pub fn exists_by_id(&self, id: &str) -> bool {
        self.verifiers.has_id(id)
    }
}
