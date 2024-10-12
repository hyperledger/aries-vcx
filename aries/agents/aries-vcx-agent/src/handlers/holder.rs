use std::sync::Arc;

use aries_vcx::{
    did_parser_nom::Did,
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
use aries_vcx_anoncreds::anoncreds::anoncreds::Anoncreds;
use aries_vcx_ledger::ledger::indy_vdr_ledger::DefaultIndyLedgerRead;
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;

use crate::{
    error::*,
    handlers::connection::ServiceConnections,
    http::VcxHttpClient,
    storage::{agent_storage_inmem::AgentStorageInMem, AgentStorage},
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

pub struct ServiceCredentialsHolder<T> {
    ledger_read: Arc<DefaultIndyLedgerRead>,
    anoncreds: Anoncreds,
    wallet: Arc<T>,
    creds_holder: AgentStorageInMem<HolderWrapper>,
    service_connections: Arc<ServiceConnections<T>>,
}

impl<T: BaseWallet> ServiceCredentialsHolder<T> {
    pub fn new(
        ledger_read: Arc<DefaultIndyLedgerRead>,
        anoncreds: Anoncreds,
        wallet: Arc<T>,
        service_connections: Arc<ServiceConnections<T>>,
    ) -> Self {
        Self {
            service_connections,
            creds_holder: AgentStorageInMem::new("creds-holder"),
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
        let holder = Holder::create_with_proposal("foobar", propose_credential)?;

        let aries_msg: AriesMessage = holder.get_proposal()?.into();
        self.service_connections
            .send_message(connection_id, &aries_msg)
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
        let holder = Holder::create_from_offer("foobar", offer)?;
        self.creds_holder.insert(
            &holder.get_thread_id()?,
            HolderWrapper::new(holder, connection_id),
        )
    }

    pub async fn send_credential_request(&self, thread_id: &str) -> AgentResult<String> {
        let connection_id = self.get_connection_id(thread_id)?;
        let connection = self.service_connections.get_by_id(&connection_id)?;
        // todo: technically doesn't need to be DID at all, and definitely need not to be pairwise
        // DID
        let pw_did_as_entropy = connection.pairwise_info().pw_did.to_string();

        let mut holder = self.get_holder(thread_id)?;
        let message = holder
            .prepare_credential_request(
                self.wallet.as_ref(),
                self.ledger_read.as_ref(),
                &self.anoncreds,
                Did::parse(pw_did_as_entropy)?,
            )
            .await?;

        self.service_connections
            .send_message(&connection_id, &message)
            .await?;

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
                self.wallet.as_ref(),
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
                            .send_message(self.wallet.as_ref(), &msg, &VcxHttpClient)
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
