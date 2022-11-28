use std::collections::HashMap;
use std::sync::Arc;

use crate::error::*;
use crate::storage::object_cache::ObjectCache;
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::handlers::proof_presentation::prover::Prover;
use aries_vcx::messages::proof_presentation::presentation_ack::PresentationAck;
use aries_vcx::messages::proof_presentation::presentation_proposal::PresentationProposalData;
use aries_vcx::messages::proof_presentation::presentation_request::PresentationRequest;
use aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;
use serde_json::Value;

use super::connection::ServiceConnections;

#[derive(Clone)]
struct ProverWrapper {
    prover: Prover,
    connection_id: String,
}

impl ProverWrapper {
    pub fn new(prover: Prover, connection_id: &str) -> Self {
        Self {
            prover,
            connection_id: connection_id.to_string(),
        }
    }
}

pub struct ServiceProver {
    profile: Arc<dyn Profile>,
    provers: ObjectCache<ProverWrapper>,
    service_connections: Arc<ServiceConnections>,
}

impl ServiceProver {
    pub fn new(
        profile: Arc<dyn Profile>,
        service_connections: Arc<ServiceConnections>,
    ) -> Self {
        Self {
            profile,
            service_connections,
            provers: ObjectCache::new("provers"),
        }
    }

    pub fn get_prover(&self, thread_id: &str) -> AgentResult<Prover> {
        let ProverWrapper { prover, .. } = self.provers.get(thread_id)?;
        Ok(prover)
    }

    pub fn get_connection_id(&self, thread_id: &str) -> AgentResult<String> {
        let ProverWrapper { connection_id, .. } = self.provers.get(thread_id)?;
        Ok(connection_id)
    }

    async fn get_credentials_for_presentation(&self, prover: &Prover, tails_dir: Option<&str>) -> AgentResult<String> {
        let credentials = prover.retrieve_credentials(&self.profile).await?;
        let credentials: HashMap<String, Value> =
            serde_json::from_str(&credentials).unwrap();

        let mut res_credentials = json!({});

        for (key, val) in credentials["attrs"].as_object().unwrap().iter() {
            let cred_array = val.as_array().unwrap();
            if !cred_array.is_empty() {
                let first_cred = &cred_array[0];
                res_credentials["attrs"][key]["credential"] = first_cred.clone();
                if let Some(tails_dir) = tails_dir {
                    res_credentials["attrs"][key]["tails_file"] = Value::from(tails_dir);
                }
            }
        }
        Ok(res_credentials.to_string())
    }

    pub fn create_from_request(
        &self,
        connection_id: &str,
        request: PresentationRequest,
    ) -> AgentResult<String> {
        self.service_connections.get_by_id(connection_id)?;
        let prover = Prover::create_from_request("", request)?;
        self.provers.set(
            &prover.get_thread_id()?,
            ProverWrapper::new(prover, connection_id),
        )
    }

    pub async fn send_proof_proposal(
        &self,
        connection_id: &str,
        proposal: PresentationProposalData,
    ) -> AgentResult<String> {
        let connection = self.service_connections.get_by_id(connection_id)?;
        let mut prover = Prover::create("")?;
        prover
            .send_proposal(
                proposal,
                connection.send_message_closure(&self.profile, None).await?,
            )
            .await?;
        self.provers.set(
            &prover.get_thread_id()?,
            ProverWrapper::new(prover, connection_id),
        )
    }

    pub fn is_secondary_proof_requested(&self, thread_id: &str) -> AgentResult<bool> {
        let prover = self.get_prover(thread_id)?;
        let attach = prover.get_proof_request_attachment()?;
        let attach: Value = serde_json::from_str(&attach)?;
        Ok(!attach["non_revoked"].is_null())
    }

    pub async fn send_proof_prentation(&self, thread_id: &str, tails_dir: Option<&str>) -> AgentResult<()> {
        let ProverWrapper {
            mut prover,
            connection_id,
        } = self.provers.get(thread_id)?;
        let connection = self.service_connections.get_by_id(&connection_id)?;
        let credentials = self.get_credentials_for_presentation(&prover, tails_dir).await?;
        prover
            .generate_presentation(
                &self.profile,
                credentials,
                "{}".to_string(),
            )
            .await?;
        prover
            .send_presentation(
                connection.send_message_closure(&self.profile, None).await?,
            )
            .await?;
        self.provers.set(
            &prover.get_thread_id()?,
            ProverWrapper::new(prover, &connection_id),
        )?;
        Ok(())
    }

    pub fn process_presentation_ack(
        &self,
        thread_id: &str,
        ack: PresentationAck,
    ) -> AgentResult<String> {
        let ProverWrapper { mut prover, connection_id } = self.provers.get(thread_id)?;
        prover.process_presentation_ack(ack)?;
        self.provers.set(
            &prover.get_thread_id()?,
            ProverWrapper::new(prover, &connection_id),
        )
    }

    pub fn get_state(&self, thread_id: &str) -> AgentResult<ProverState> {
        let ProverWrapper { prover, .. } = self.provers.get(thread_id)?;
        Ok(prover.get_state())
    }

    pub fn exists_by_id(&self, thread_id: &str) -> bool {
        self.provers.has_id(thread_id)
    }
}
