use std::sync::Arc;

use aries_vcx::{
    handlers::issuance::holder::Holder,
    messages::{
        msg_fields::protocols::cred_issuance::v1::{
            issue_credential::IssueCredentialV1, offer_credential::OfferCredentialV1,
            propose_credential::ProposeCredentialV1,
        },
        AriesMessage,
    },
    protocols::{issuance::holder::state_machine::HolderState, SendClosure},
};
use aries_vcx_core::{
    anoncreds::credx_anoncreds::IndyCredxAnonCreds,
    ledger::indy_vdr_ledger::DefaultIndyLedgerRead,
    wallet::{base_wallet::BaseWallet, indy::IndySdkWallet},
};

use crate::{
    error::*,
    http::VcxHttpClient,
    services::connection::ServiceConnections,
    storage::{object_cache::ObjectCache, Storage},
};

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
    ledger_read: Arc<DefaultIndyLedgerRead>,
    anoncreds: IndyCredxAnonCreds,
    wallet: Arc<dyn BaseWallet>,
    creds_holder: ObjectCache<HolderWrapper>,
    service_connections: Arc<ServiceConnections>,
}

impl ServiceCredentialsHolder {
    pub fn new(
        ledger_read: Arc<DefaultIndyLedgerRead>,
        anoncreds: IndyCredxAnonCreds,
        wallet: Arc<dyn BaseWallet>,
        service_connections: Arc<ServiceConnections>,
    ) -> Self {
        Self {
            service_connections,
            creds_holder: ObjectCache::new("creds-holder"),
            ledger_read,
            anoncreds,
            wallet,
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
        propose_credential: ProposeCredentialV1,
    ) -> AgentResult<String> {
        let connection = self.service_connections.get_by_id(connection_id)?;

        let mut holder = Holder::create("")?;
        holder.set_proposal(propose_credential.clone())?;
        connection
            .send_message(&self.wallet, &propose_credential.into(), &VcxHttpClient)
            .await?;

        self.creds_holder.insert(
            &holder.get_thread_id()?,
            HolderWrapper::new(holder, connection_id),
        )
    }

    pub fn create_from_offer(
        &self,
        connection_id: &str,
        offer: OfferCredentialV1,
    ) -> AgentResult<String> {
        self.service_connections.get_by_id(connection_id)?;
        let holder = Holder::create_from_offer("", offer)?;
        self.creds_holder.insert(
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

        let pw_did = connection.pairwise_info().pw_did.to_string();

        let send_closure: SendClosure = Box::new(|msg: AriesMessage| {
            Box::pin(async move {
                connection
                    .send_message(&self.wallet, &msg, &VcxHttpClient)
                    .await
            })
        });
        let msg_response = holder
            .prepare_credential_request(
                &self.wallet,
                self.ledger_read.as_ref(),
                &self.anoncreds,
                pw_did,
            )
            .await?;
        send_closure(msg_response).await?;
        self.creds_holder.insert(
            &holder.get_thread_id()?,
            HolderWrapper::new(holder, &connection_id),
        )
    }

    pub async fn process_credential(
        &self,
        thread_id: &str,
        msg_issue_credential: IssueCredentialV1,
    ) -> AgentResult<String> {
        let mut holder = self.get_holder(thread_id)?;
        let connection_id = self.get_connection_id(thread_id)?;
        let connection = self.service_connections.get_by_id(&connection_id)?;

        holder
            .process_credential(
                &self.wallet,
                self.ledger_read.as_ref(),
                &self.anoncreds,
                msg_issue_credential.clone(),
            )
            .await?;
        match holder.get_final_message()? {
            None => {}
            Some(msg_response) => {
                let send_closure: SendClosure = Box::new(|msg: AriesMessage| {
                    Box::pin(async move {
                        connection
                            .send_message(&self.wallet, &msg, &VcxHttpClient)
                            .await
                    })
                });
                send_closure(msg_response).await?;
            }
        }
        self.creds_holder.insert(
            &holder.get_thread_id()?,
            HolderWrapper::new(holder, &connection_id),
        )
    }

    pub fn get_state(&self, thread_id: &str) -> AgentResult<HolderState> {
        Ok(self.get_holder(thread_id)?.get_state())
    }

    pub async fn is_revokable(&self, thread_id: &str) -> AgentResult<bool> {
        self.get_holder(thread_id)?
            .is_revokable(self.ledger_read.as_ref())
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
        self.creds_holder.contains_key(thread_id)
    }
}
