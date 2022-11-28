use std::sync::Arc;

use crate::error::*;
use crate::storage::object_cache::ObjectCache;
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::handlers::proof_presentation::verifier::Verifier;
use aries_vcx::messages::proof_presentation::presentation::Presentation;
use aries_vcx::messages::proof_presentation::presentation_proposal::PresentationProposal;
use aries_vcx::messages::status::Status;
use aries_vcx::protocols::proof_presentation::verifier::state_machine::VerifierState;
use aries_vcx::xyz::proofs::proof_request::PresentationRequestData;

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
    profile: Arc<dyn Profile>,
    verifiers: ObjectCache<VerifierWrapper>,
    service_connections: Arc<ServiceConnections>,
}

impl ServiceVerifier {
    pub fn new(
        profile: Arc<dyn Profile>,
        service_connections: Arc<ServiceConnections>,
    ) -> Self {
        Self {
            profile,
            service_connections,
            verifiers: ObjectCache::new("verifiers"),
        }
    }

    pub async fn send_proof_request(
        &self,
        connection_id: &str,
        request: PresentationRequestData,
        proposal: Option<PresentationProposal>,
    ) -> AgentResult<String> {
        let connection = self.service_connections.get_by_id(connection_id)?;
        let mut verifier = if let Some(proposal) = proposal {
            Verifier::create_from_proposal("", &proposal)?
        } else {
            Verifier::create_from_request("".to_string(), &request)?
        };
        verifier
            .send_presentation_request(connection.send_message_closure(&self.profile, None).await?)
            .await?;
        self.verifiers.set(
            &verifier.get_thread_id()?,
            VerifierWrapper::new(verifier, connection_id),
        )
    }

    pub fn get_presentation_status(&self, thread_id: &str) -> AgentResult<Status> {
        let VerifierWrapper { verifier, .. } = self.verifiers.get(thread_id)?;
        Ok(verifier.get_presentation_status())
    }

    pub async fn verify_presentation(&self, thread_id: &str, presentation: Presentation) -> AgentResult<()> {
        let VerifierWrapper { mut verifier, connection_id } = self.verifiers.get(thread_id)?;
        let connection = self.service_connections.get_by_id(&connection_id)?;
        verifier.verify_presentation(&self.profile, presentation, connection.send_message_closure(&self.profile, None).await?).await?;
        self.verifiers.set(
            thread_id,
            VerifierWrapper::new(verifier, &connection_id),
        )?;
        Ok(())
    }

    pub fn get_state(&self, thread_id: &str) -> AgentResult<VerifierState> {
        let VerifierWrapper { verifier, .. } = self.verifiers.get(thread_id)?;
        Ok(verifier.get_state())
    }

    pub fn exists_by_id(&self, thread_id: &str) -> bool {
        self.verifiers.has_id(thread_id)
    }
}
