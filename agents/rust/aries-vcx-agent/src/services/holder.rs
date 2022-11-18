use std::sync::Arc;

use crate::error::*;
use crate::services::connection::ServiceConnections;
use crate::storage::object_cache::ObjectCache;
use aries_vcx::handlers::issuance::holder::Holder;
use aries_vcx::messages::issuance::credential::Credential;
use aries_vcx::messages::issuance::credential_offer::CredentialOffer;
use aries_vcx::messages::issuance::credential_proposal::CredentialProposalData;
use aries_vcx::protocols::issuance::holder::state_machine::HolderState;
use aries_vcx::vdrtools::{PoolHandle, WalletHandle};

#[derive(Clone)]
struct HolderWrapper {
    holder: Holder,
    connection_id: String,
}

impl HolderWrapper {
    pub fn new(holder: Holder, connection_id: &str) -> Self {
        Self {
            holder,
            connection_id: connection_id.to_string(),
        }
    }
}

pub struct ServiceCredentialsHolder {
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    creds_holder: ObjectCache<HolderWrapper>,
    service_connections: Arc<ServiceConnections>,
}

impl ServiceCredentialsHolder {
    pub fn new(
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        service_connections: Arc<ServiceConnections>,
    ) -> Self {
        Self {
            wallet_handle,
            pool_handle,
            service_connections,
            creds_holder: ObjectCache::new("creds-holder"),
        }
    }

    fn get_holder(&self, thread_id: &str) -> AgentResult<Holder> {
        let HolderWrapper { holder, .. } = self.creds_holder.get(thread_id)?;
        Ok(holder)
    }

    pub fn get_connection_id(&self, thread_id: &str) -> AgentResult<String> {
        let HolderWrapper { connection_id, .. } = self.creds_holder.get(thread_id)?;
        Ok(connection_id)
    }

    pub async fn send_credential_proposal(
        &self,
        connection_id: &str,
        proposal_data: CredentialProposalData,
    ) -> AgentResult<String> {
        let connection = self.service_connections.get_by_id(connection_id)?;
        let mut holder = Holder::create("")?;
        holder
            .send_proposal(
                proposal_data,
                connection.send_message_closure(self.wallet_handle, None).await?,
            )
            .await?;
        self.creds_holder.set(
            &holder.get_thread_id()?,
            HolderWrapper::new(holder, connection_id),
        )
    }

    pub fn create_from_offer(
        &self,
        connection_id: &str,
        offer: CredentialOffer,
    ) -> AgentResult<String> {
        self.service_connections.get_by_id(connection_id)?;
        let holder = Holder::create_from_offer("", offer)?;
        self.creds_holder.set(
            &holder.get_thread_id()?,
            HolderWrapper::new(holder, connection_id),
        )
    }

    pub async fn send_credential_request(
        &self,
        thread_id: Option<&str>,
        connection_id: Option<&str>,
    ) -> AgentResult<String> {
        let (mut holder, connection_id) = match (thread_id, connection_id) {
            (Some(id), Some(connection_id)) => (self.get_holder(id)?, connection_id.to_string()),
            (Some(id), None) => (self.get_holder(id)?, self.get_connection_id(id)?),
            (None, Some(connection_id)) => (Holder::create("")?, connection_id.to_string()),
            (None, None) => return Err(AgentError::from_kind(AgentErrorKind::InvalidArguments)),
        };
        let connection = self.service_connections.get_by_id(&connection_id)?;
        holder
            .send_request(
                self.wallet_handle,
                self.pool_handle,
                connection.pairwise_info().pw_did.to_string(),
                connection.send_message_closure(self.wallet_handle, None).await?,
            )
            .await?;
        self.creds_holder.set(
            &holder.get_thread_id()?,
            HolderWrapper::new(holder, &connection_id),
        )
    }

    pub async fn process_credential(
        &self,
        thread_id: &str,
        credential: Credential,
    ) -> AgentResult<String> {
        let mut holder = self.get_holder(thread_id)?;
        let connection_id = self.get_connection_id(thread_id)?;
        let connection = self.service_connections.get_by_id(&connection_id)?;
        holder
            .process_credential(
                self.wallet_handle,
                self.pool_handle,
                credential,
                connection.send_message_closure(self.wallet_handle, None).await?,
            )
            .await?;
        self.creds_holder.set(
            &holder.get_thread_id()?,
            HolderWrapper::new(holder, &connection_id),
        )
    }

    pub fn get_state(&self, thread_id: &str) -> AgentResult<HolderState> {
        Ok(self.get_holder(thread_id)?.get_state())
    }

    pub async fn is_revokable(&self, thread_id: &str) -> AgentResult<bool> {
        self.get_holder(thread_id)?
            .is_revokable(self.wallet_handle, self.pool_handle)
            .await
            .map_err(|err| err.into())
    }

    pub async fn get_rev_reg_id(&self, thread_id: &str) -> AgentResult<String> {
        self.get_holder(thread_id)?
            .get_rev_reg_id()
            .map_err(|err| err.into())
    }

    pub async fn get_tails_hash(&self, thread_id: &str) -> AgentResult<String> {
        self.get_holder(thread_id)?
            .get_tails_hash()
            .map_err(|err| err.into())
    }

    pub async fn get_tails_location(&self, thread_id: &str) -> AgentResult<String> {
        self.get_holder(thread_id)?
            .get_tails_location()
            .map_err(|err| err.into())
    }

    pub fn exists_by_id(&self, thread_id: &str) -> bool {
        self.creds_holder.has_id(thread_id)
    }
}
