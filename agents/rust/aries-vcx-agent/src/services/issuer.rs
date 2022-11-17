use std::sync::Arc;

use crate::error::*;
use crate::services::connection::ServiceConnections;
use crate::storage::object_cache::ObjectCache;
use aries_vcx::handlers::issuance::issuer::Issuer;
use aries_vcx::messages::issuance::credential_offer::OfferInfo;
use aries_vcx::messages::issuance::credential_proposal::CredentialProposal;
use aries_vcx::protocols::issuance::issuer::state_machine::IssuerState;
use aries_vcx::vdrtools_sys::WalletHandle;

#[derive(Clone)]
struct IssuerWrapper {
    issuer: Issuer,
    connection_id: String,
}

impl IssuerWrapper {
    pub fn new(issuer: Issuer, connection_id: &str) -> Self {
        Self {
            issuer,
            connection_id: connection_id.to_string(),
        }
    }
}

pub struct ServiceCredentialsIssuer {
    wallet_handle: WalletHandle,
    creds_issuer: ObjectCache<IssuerWrapper>,
    service_connections: Arc<ServiceConnections>,
}

impl ServiceCredentialsIssuer {
    pub fn new(
        wallet_handle: WalletHandle,
        service_connections: Arc<ServiceConnections>,
    ) -> Self {
        Self {
            wallet_handle,
            service_connections,
            creds_issuer: ObjectCache::new("creds-issuer"),
        }
    }

    fn get_issuer(&self, thread_id: &str) -> AgentResult<Issuer> {
        let IssuerWrapper { issuer, .. } = self.creds_issuer.get(thread_id)?;
        Ok(issuer)
    }

    pub fn get_connection_id(&self, thread_id: &str) -> AgentResult<String> {
        let IssuerWrapper { connection_id, .. } = self.creds_issuer.get(thread_id)?;
        Ok(connection_id)
    }

    pub async fn accept_proposal(
        &self,
        connection_id: &str,
        proposal: &CredentialProposal,
    ) -> AgentResult<String> {
        let issuer = Issuer::create_from_proposal("", proposal)?;
        self.creds_issuer.set(
            &issuer.get_thread_id()?,
            IssuerWrapper::new(issuer, connection_id),
        )
    }

    pub async fn send_credential_offer(
        &self,
        thread_id: Option<&str>,
        connection_id: Option<&str>,
        offer_info: OfferInfo,
    ) -> AgentResult<String> {
        let (mut issuer, connection_id) = match (thread_id, connection_id) {
            (Some(id), Some(connection_id)) => (self.get_issuer(id)?, connection_id.to_string()),
            (Some(id), None) => (self.get_issuer(id)?, self.get_connection_id(id)?),
            (None, Some(connection_id)) => (Issuer::create("")?, connection_id.to_string()),
            (None, None) => return Err(AgentError::from_kind(AgentErrorKind::InvalidArguments)),
        };
        let connection = self.service_connections.get_by_id(&connection_id)?;
        issuer
            .build_credential_offer_msg(self.wallet_handle, offer_info, None)
            .await?;
        issuer
            .send_credential_offer(connection.send_message_closure(self.wallet_handle, None).await?)
            .await?;
        self.creds_issuer.set(
            &issuer.get_thread_id()?,
            IssuerWrapper::new(issuer, &connection_id),
        )
    }

    pub async fn send_credential(&self, thread_id: &str) -> AgentResult<()> {
        let IssuerWrapper {
            mut issuer,
            connection_id,
        } = self.creds_issuer.get(thread_id)?;
        let connection = self.service_connections.get_by_id(&connection_id)?;
        issuer
            .send_credential(
                self.wallet_handle,
                connection.send_message_closure(self.wallet_handle, None).await?,
            )
            .await?;
        self.creds_issuer.set(
            &issuer.get_thread_id()?,
            IssuerWrapper::new(issuer, &connection_id),
        )?;
        Ok(())
    }

    pub fn get_state(&self, thread_id: &str) -> AgentResult<IssuerState> {
        Ok(self.get_issuer(thread_id)?.get_state())
    }

    pub fn get_rev_reg_id(&self, thread_id: &str) -> AgentResult<String> {
        let issuer = self.get_issuer(thread_id)?;
        issuer.get_rev_reg_id().map_err(|err| err.into())
    }

    pub fn get_rev_id(&self, thread_id: &str) -> AgentResult<String> {
        let issuer = self.get_issuer(thread_id)?;
        issuer.get_rev_id().map_err(|err| err.into())
    }

    pub fn exists_by_id(&self, thread_id: &str) -> bool {
        self.creds_issuer.has_id(thread_id)
    }
}
