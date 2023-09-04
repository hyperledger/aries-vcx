use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use messages::decorators::thread::Thread;
use messages::decorators::timing::Timing;
use messages::msg_fields::protocols::cred_issuance::ack::{AckCredential, AckCredentialContent};
use messages::msg_fields::protocols::cred_issuance::issue_credential::IssueCredential;
use messages::msg_fields::protocols::cred_issuance::offer_credential::OfferCredential;
use messages::msg_fields::protocols::cred_issuance::propose_credential::ProposeCredential;
use messages::msg_fields::protocols::cred_issuance::request_credential::RequestCredential;
use messages::msg_fields::protocols::cred_issuance::CredentialIssuance;
use messages::msg_fields::protocols::notification::ack::{AckDecorators, AckStatus};
use messages::msg_fields::protocols::report_problem::ProblemReport;
use messages::msg_fields::protocols::revocation::revoke::Revoke;
use messages::AriesMessage;

use crate::common::credentials::get_cred_rev_id;
use crate::errors::error::prelude::*;
use crate::handlers::connection::mediated_connection::MediatedConnection;
use crate::handlers::revocation_notification::receiver::RevocationNotificationReceiver;
use crate::protocols::issuance::holder::state_machine::{HolderFullState, HolderSM, HolderState};

fn build_credential_ack(thread_id: &str) -> AckCredential {
    let content = AckCredentialContent::new(AckStatus::Ok);
    let mut decorators = AckDecorators::new(Thread::new(thread_id.to_owned()));
    let mut timing = Timing::default();
    timing.out_time = Some(Utc::now());
    decorators.timing = Some(timing);

    AckCredential::with_decorators(Uuid::new_v4().to_string(), content, decorators)
}

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

    pub fn create_with_proposal(source_id: &str, propose_credential: ProposeCredential) -> VcxResult<Holder> {
        trace!(
            "Holder::create_with_proposal >>> source_id: {:?}, propose_credential: {:?}",
            source_id,
            propose_credential
        );
        let holder_sm = HolderSM::with_proposal(propose_credential, source_id.to_string());
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

    pub fn set_proposal(&mut self, credential_proposal: ProposeCredential) -> VcxResult<()> {
        self.holder_sm = self.holder_sm.clone().set_proposal(credential_proposal)?;
        Ok(())
    }

    pub async fn prepare_credential_request(
        &mut self,
        ledger: &Arc<dyn AnoncredsLedgerRead>,
        anoncreds: &Arc<dyn BaseAnonCreds>,
        my_pw_did: String,
    ) -> VcxResult<AriesMessage> {
        self.holder_sm = self
            .holder_sm
            .clone()
            .prepare_credential_request(ledger, anoncreds, my_pw_did)
            .await?;
        match self.get_state() {
            HolderState::Failed => {
                Ok(self.get_problem_report()?.into())
            }
            HolderState::RequestSet => {
                Ok(self.get_msg_credential_request()?.into())
            }
            _ => {
                Err(AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, "Holder::prepare_credential_request >> reached unexpected state after calling prepare_credential_request"))
            }
        }
    }

    pub fn get_msg_credential_request(&self) -> VcxResult<RequestCredential> {
        match self.holder_sm.state {
            HolderFullState::RequestSet(ref state) => {
                let mut msg: RequestCredential = state.msg_credential_request.clone().into();
                let mut timing = Timing::default();
                timing.out_time = Some(Utc::now());
                msg.decorators.timing = Some(timing);
                Ok(msg)
            }
            _ => Err(AriesVcxError::from_msg(AriesVcxErrorKind::NotReady, "Invalid action")),
        }
    }

    pub fn decline_offer<'a>(&'a mut self, comment: Option<&'a str>) -> VcxResult<ProblemReport> {
        self.holder_sm = self.holder_sm.clone().decline_offer(comment.map(String::from))?;
        self.get_problem_report()
    }

    pub async fn process_credential(
        &mut self,
        ledger: &Arc<dyn AnoncredsLedgerRead>,
        anoncreds: &Arc<dyn BaseAnonCreds>,
        credential: IssueCredential,
    ) -> VcxResult<()> {
        self.holder_sm = self
            .holder_sm
            .clone()
            .receive_credential(ledger, anoncreds, credential)
            .await?;
        Ok(())
    }

    pub fn is_terminal_state(&self) -> bool {
        self.holder_sm.is_terminal_state()
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

    pub fn get_problem_report(&self) -> VcxResult<ProblemReport> {
        self.holder_sm.get_problem_report()
    }

    // todo 0109: send ack/problem-report in upper layer
    pub async fn process_aries_msg(
        &mut self,
        ledger: &Arc<dyn AnoncredsLedgerRead>,
        anoncreds: &Arc<dyn BaseAnonCreds>,
        message: AriesMessage,
    ) -> VcxResult<()> {
        let holder_sm = match message {
            AriesMessage::CredentialIssuance(CredentialIssuance::OfferCredential(offer)) => {
                self.holder_sm.clone().receive_offer(offer)?
            }
            AriesMessage::CredentialIssuance(CredentialIssuance::IssueCredential(credential)) => {
                self.holder_sm
                    .clone()
                    .receive_credential(ledger, anoncreds, credential)
                    .await?
            }
            // TODO: What about credential issuance problem report?
            AriesMessage::ReportProblem(report) => self.holder_sm.clone().receive_problem_report(report)?,
            _ => self.holder_sm.clone(),
        };
        self.holder_sm = holder_sm;
        Ok(())
    }

    pub fn get_final_message(&self) -> VcxResult<Option<AriesMessage>> {
        match &self.holder_sm.state {
            HolderFullState::Finished(state) => {
                if let Some(ack_requested) = state.ack_requested {
                    if ack_requested {
                        let ack_msg = build_credential_ack(&self.get_thread_id()?);
                        return Ok(Some(ack_msg.into()));
                    }
                }
            }
            _ => {}
        };
        return Ok(None);
    }
}
