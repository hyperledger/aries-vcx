use std::collections::HashMap;

use messages::msg_fields::protocols::cred_issuance::issue_credential::IssueCredential;
use messages::msg_fields::protocols::cred_issuance::offer_credential::OfferCredential;
use messages::msg_fields::protocols::cred_issuance::propose_credential::ProposeCredential;
use messages::msg_fields::protocols::revocation::revoke::Revoke;
use messages::AriesMessage;
use std::sync::Arc;

use agency_client::agency_client::AgencyClient;
use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use aries_vcx_core::wallet::base_wallet::BaseWallet;

use crate::common::credentials::get_cred_rev_id;
use crate::errors::error::prelude::*;
use crate::handlers::connection::mediated_connection::MediatedConnection;
use crate::handlers::issuance::mediated_holder::holder_find_message_to_handle;
use crate::handlers::revocation_notification::receiver::RevocationNotificationReceiver;
use crate::protocols::issuance::actions::CredentialIssuanceAction;
use crate::protocols::issuance::holder::state_machine::{HolderSM, HolderState};
use crate::protocols::SendClosure;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Holder {
    holder_sm: HolderSM,
}

impl Holder {
    pub fn create(source_id: &str) -> VcxResult<Holder> {
        trace!("Holder::create >>> source_id: {:?}", source_id);
        let holder_sm = HolderSM::new(source_id.to_string());
        Ok(Holder { holder_sm })
    }

    pub fn create_from_offer(source_id: &str, credential_offer: OfferCredential) -> VcxResult<Holder> {
        trace!(
            "Holder::create_from_offer >>> source_id: {:?}, credential_offer: {:?}",
            source_id,
            credential_offer
        );
        let holder_sm = HolderSM::from_offer(credential_offer, source_id.to_string());
        Ok(Holder { holder_sm })
    }

    pub async fn send_proposal(
        &mut self,
        credential_proposal: ProposeCredential,
        send_message: SendClosure,
    ) -> VcxResult<()> {
        self.holder_sm = self
            .holder_sm
            .clone()
            .send_proposal(credential_proposal, send_message)
            .await?;
        Ok(())
    }

    pub async fn send_request(
        &mut self,
        ledger: &Arc<dyn AnoncredsLedgerRead>,
        anoncreds: &Arc<dyn BaseAnonCreds>,
        my_pw_did: String,
        send_message: SendClosure,
    ) -> VcxResult<()> {
        self.holder_sm = self
            .holder_sm
            .clone()
            .send_request(ledger, anoncreds, my_pw_did, send_message)
            .await?;
        Ok(())
    }

    pub async fn decline_offer<'a>(&'a mut self, comment: Option<&'a str>, send_message: SendClosure) -> VcxResult<()> {
        self.holder_sm = self
            .holder_sm
            .clone()
            .decline_offer(comment.map(String::from), send_message)
            .await?;
        Ok(())
    }

    pub async fn process_credential(
        &mut self,
        ledger: &Arc<dyn AnoncredsLedgerRead>,
        anoncreds: &Arc<dyn BaseAnonCreds>,
        credential: IssueCredential,
        send_message: SendClosure,
    ) -> VcxResult<()> {
        self.holder_sm = self
            .holder_sm
            .clone()
            .receive_credential(ledger, anoncreds, credential, send_message)
            .await?;
        Ok(())
    }

    pub fn is_terminal_state(&self) -> bool {
        self.holder_sm.is_terminal_state()
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, AriesMessage>) -> Option<(String, AriesMessage)> {
        holder_find_message_to_handle(&self.holder_sm, messages)
    }

    pub fn get_state(&self) -> HolderState {
        self.holder_sm.get_state()
    }

    pub fn get_source_id(&self) -> String {
        self.holder_sm.get_source_id()
    }

    pub fn get_credential(&self) -> VcxResult<(String, AriesMessage)> {
        self.holder_sm.get_credential()
    }

    pub fn get_attributes(&self) -> VcxResult<String> {
        self.holder_sm.get_attributes()
    }

    pub fn get_attachment(&self) -> VcxResult<String> {
        self.holder_sm.get_attachment()
    }

    pub fn get_offer(&self) -> VcxResult<OfferCredential> {
        self.holder_sm.get_offer()
    }

    pub fn get_tails_location(&self) -> VcxResult<String> {
        self.holder_sm.get_tails_location()
    }

    pub fn get_tails_hash(&self) -> VcxResult<String> {
        self.holder_sm.get_tails_hash()
    }

    pub fn get_rev_reg_id(&self) -> VcxResult<String> {
        self.holder_sm.get_rev_reg_id()
    }

    pub fn get_cred_id(&self) -> VcxResult<String> {
        self.holder_sm.get_cred_id()
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        self.holder_sm.get_thread_id()
    }

    pub async fn is_revokable(&self, ledger: &Arc<dyn AnoncredsLedgerRead>) -> VcxResult<bool> {
        self.holder_sm.is_revokable(ledger).await
    }

    pub async fn is_revoked(
        &self,
        ledger: &Arc<dyn AnoncredsLedgerRead>,
        anoncreds: &Arc<dyn BaseAnonCreds>,
    ) -> VcxResult<bool> {
        self.holder_sm.is_revoked(ledger, anoncreds).await
    }

    pub async fn delete_credential(&self, anoncreds: &Arc<dyn BaseAnonCreds>) -> VcxResult<()> {
        self.holder_sm.delete_credential(anoncreds).await
    }

    pub fn get_credential_status(&self) -> VcxResult<u32> {
        Ok(self.holder_sm.credential_status())
    }

    pub async fn get_cred_rev_id(&self, anoncreds: &Arc<dyn BaseAnonCreds>) -> VcxResult<String> {
        get_cred_rev_id(anoncreds, &self.get_cred_id()?).await
    }

    pub async fn handle_revocation_notification(
        &self,
        ledger: &Arc<dyn AnoncredsLedgerRead>,
        anoncreds: &Arc<dyn BaseAnonCreds>,
        wallet: &Arc<dyn BaseWallet>,
        connection: &MediatedConnection,
        notification: Revoke,
    ) -> VcxResult<()> {
        if self.holder_sm.is_revokable(ledger).await? {
            let send_message = connection.send_message_closure(Arc::clone(wallet)).await?;
            // TODO: Store to remember notification was received along with details
            RevocationNotificationReceiver::build(self.get_rev_reg_id()?, self.get_cred_rev_id(anoncreds).await?)
                .handle_revocation_notification(notification, send_message)
                .await?;
            Ok(())
        } else {
            Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Unexpected revocation notification, credential is not revokable".to_string(),
            ))
        }
    }

    pub async fn step(
        &mut self,
        ledger: &Arc<dyn AnoncredsLedgerRead>,
        anoncreds: &Arc<dyn BaseAnonCreds>,
        message: CredentialIssuanceAction,
        send_message: Option<SendClosure>,
    ) -> VcxResult<()> {
        self.holder_sm = self
            .holder_sm
            .clone()
            .handle_message(ledger, anoncreds, message, send_message)
            .await?;
        Ok(())
    }

    pub async fn update_state(
        &mut self,
        ledger: &Arc<dyn AnoncredsLedgerRead>,
        anoncreds: &Arc<dyn BaseAnonCreds>,
        wallet: &Arc<dyn BaseWallet>,
        agency_client: &AgencyClient,
        connection: &MediatedConnection,
    ) -> VcxResult<HolderState> {
        trace!("Holder::update_state >>>");
        if self.is_terminal_state() {
            return Ok(self.get_state());
        }
        let send_message = connection.send_message_closure(Arc::clone(wallet)).await?;

        let messages = connection.get_messages(agency_client).await?;
        if let Some((uid, msg)) = self.find_message_to_handle(messages) {
            self.step(ledger, anoncreds, msg.into(), Some(send_message)).await?;
            connection.update_message_status(&uid, agency_client).await?;
        }
        Ok(self.get_state())
    }
}
