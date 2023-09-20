use std::sync::Arc;

use aries_vcx::{
    core::profile::profile::Profile,
    handlers::{issuance::issuer::Issuer, util::OfferInfo},
    messages::{
        msg_fields::protocols::cred_issuance::{
            ack::AckCredential, propose_credential::ProposeCredential,
            request_credential::RequestCredential,
        },
        AriesMessage,
    },
    protocols::{issuance::issuer::state_machine::IssuerState, SendClosure},
};

use crate::{
    error::*,
    http_client::HttpClient,
    services::connection::ServiceConnections,
    storage::{object_cache::ObjectCache, Storage},
};

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
    profile: Arc<dyn Profile>,
    creds_issuer: ObjectCache<IssuerWrapper>,
    service_connections: Arc<ServiceConnections>,
}

impl ServiceCredentialsIssuer {
    pub fn new(profile: Arc<dyn Profile>, service_connections: Arc<ServiceConnections>) -> Self {
        Self {
            profile,
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
        proposal: &ProposeCredential,
    ) -> AgentResult<String> {
        let issuer = Issuer::create_from_proposal("", proposal)?;
        self.creds_issuer.insert(
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
            .build_credential_offer_msg(&self.profile.inject_anoncreds(), offer_info, None)
            .await?;

        let wallet = self.profile.inject_wallet();

        let send_closure: SendClosure = Box::new(|msg: AriesMessage| {
            Box::pin(async move { connection.send_message(&wallet, &msg, &HttpClient).await })
        });

        let credential_offer = issuer.get_credential_offer_msg()?;
        send_closure(credential_offer).await?;
        self.creds_issuer.insert(
            &issuer.get_thread_id()?,
            IssuerWrapper::new(issuer, &connection_id),
        )
    }

    pub fn process_credential_request(
        &self,
        thread_id: &str,
        request: RequestCredential,
    ) -> AgentResult<()> {
        let IssuerWrapper {
            mut issuer,
            connection_id,
        } = self.creds_issuer.get(thread_id)?;
        issuer.process_credential_request(request)?;
        self.creds_issuer.insert(
            &issuer.get_thread_id()?,
            IssuerWrapper::new(issuer, &connection_id),
        )?;
        Ok(())
    }

    pub fn process_credential_ack(&self, thread_id: &str, ack: AckCredential) -> AgentResult<()> {
        let IssuerWrapper {
            mut issuer,
            connection_id,
        } = self.creds_issuer.get(thread_id)?;
        issuer.process_credential_ack(ack)?;
        self.creds_issuer.insert(
            &issuer.get_thread_id()?,
            IssuerWrapper::new(issuer, &connection_id),
        )?;
        Ok(())
    }

    pub async fn send_credential(&self, thread_id: &str) -> AgentResult<()> {
        let IssuerWrapper {
            mut issuer,
            connection_id,
        } = self.creds_issuer.get(thread_id)?;
        let connection = self.service_connections.get_by_id(&connection_id)?;

        let wallet = self.profile.inject_wallet();

        let send_closure: SendClosure = Box::new(|msg: AriesMessage| {
            Box::pin(async move { connection.send_message(&wallet, &msg, &HttpClient).await })
        });

        issuer
            .build_credential(&self.profile.inject_anoncreds())
            .await?;
        match issuer.get_state() {
            IssuerState::Failed => {
                let problem_report = issuer.get_problem_report()?;
                send_closure(problem_report.into()).await?;
            }
            _ => {
                let msg_issue_credential = issuer.get_msg_issue_credential()?;
                send_closure(msg_issue_credential.into()).await?;
            }
        }
        self.creds_issuer.insert(
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

    pub fn get_proposal(&self, thread_id: &str) -> AgentResult<ProposeCredential> {
        let issuer = self.get_issuer(thread_id)?;
        issuer.get_proposal().map_err(|err| err.into())
    }

    pub fn exists_by_id(&self, thread_id: &str) -> bool {
        self.creds_issuer.contains_key(thread_id)
    }
}
