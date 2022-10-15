use std::sync::Arc;

use crate::error::*;
use crate::storage::in_memory::ObjectCache;
use aries_vcx::agency_client::agency_client::AgencyClient;
use aries_vcx::agency_client::configuration::AgencyClientConfig;
use aries_vcx::handlers::proof_presentation::prover::Prover;
use aries_vcx::messages::proof_presentation::presentation_proposal::PresentationProposalData;
use aries_vcx::messages::proof_presentation::presentation_request::PresentationRequest;
use aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;
use aries_vcx::vdrtools_sys::{PoolHandle, WalletHandle};

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
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    config_agency_client: AgencyClientConfig,
    provers: ObjectCache<ProverWrapper>,
    service_connections: Arc<ServiceConnections>,
}

impl ServiceProver {
    pub fn new(wallet_handle: WalletHandle, pool_handle: PoolHandle, config_agency_client: AgencyClientConfig, service_connections: Arc<ServiceConnections>) -> Self {
        Self {
            wallet_handle,
            pool_handle,
            config_agency_client,
            service_connections,
            provers: ObjectCache::new("provers"),
        }
    }

    pub fn get_connection_id(&self, id: &str) -> AgentResult<String> {
        let ProverWrapper { connection_id, .. } = self.provers.get_cloned(id)?;
        Ok(connection_id)
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

    async fn get_credentials_for_presentation(&self, prover: &Prover) -> AgentResult<String> {
        let credentials = prover.retrieve_credentials(self.wallet_handle).await?;
        let credentials: std::collections::HashMap<String, serde_json::Value> =
            serde_json::from_str(&credentials).unwrap();

        let mut res_credentials = json!({});

        for (referent, credentials) in credentials["attrs"].as_object().unwrap().iter() {
            res_credentials["attrs"][referent] = json!({
                "credential": credentials[0]
            })
        }

        Ok(res_credentials.to_string())
    }

    pub fn create_from_request(&self, connection_id: &str, request: PresentationRequest) -> AgentResult<String> {
        self.service_connections.get_by_id(&connection_id)?;
        let prover = Prover::create_from_request("", request)?;
        self.provers.add(&prover.get_thread_id()?, ProverWrapper::new(prover, connection_id))
    }

    pub async fn send_proof_proposal(&self, connection_id: &str, proposal: PresentationProposalData) -> AgentResult<String> {
        let connection = self.service_connections.get_by_id(&connection_id)?;
        let mut prover = Prover::create("")?;
        prover
            .send_proposal(
                self.wallet_handle,
                self.pool_handle,
                proposal,
                connection
                    .send_message_closure(self.wallet_handle)
                    .await?,
            )
            .await?;
        self.provers.add(&prover.get_thread_id()?, ProverWrapper::new(prover, connection_id))
    }

    pub async fn send_proof_prentation(&self, id: &str) -> AgentResult<()> {
        let ProverWrapper { mut prover, connection_id } = self.provers.get_cloned(id)?;
        let connection = self.service_connections.get_by_id(&connection_id)?;
        let credentials = self.get_credentials_for_presentation(&prover).await?;
        prover
            .generate_presentation(
                self.wallet_handle,
                self.pool_handle,
                credentials,
                "{}".to_string(),
            )
            .await?;
        prover
            .send_presentation(
                self.wallet_handle,
                self.pool_handle,
                connection
                    .send_message_closure(self.wallet_handle)
                    .await?,
            )
            .await?;
        self.provers.add(&prover.get_thread_id()?, ProverWrapper::new(prover, &connection_id))?;
        Ok(())
    }

    pub async fn update_state(&self, id: &str) -> AgentResult<ProverState> {
        let ProverWrapper { mut prover, connection_id } = self.provers.get_cloned(id)?;
        let connection = self.service_connections.get_by_id(&connection_id)?;
        let state = prover
            .update_state(
                self.wallet_handle,
                self.pool_handle,
                &self.agency_client()?,
                &connection
            )
            .await?;
        self.provers.add(&prover.get_thread_id()?, ProverWrapper::new(prover, &connection_id))?;
        Ok(state)
    }

    pub fn get_state(&self, id: &str) -> AgentResult<ProverState> {
        let ProverWrapper { prover, .. } = self.provers.get_cloned(id)?;
        Ok(prover.get_state())
    }

    pub fn exists_by_id(&self, id: &str) -> bool {
        self.provers.has_id(id)
    }
}
