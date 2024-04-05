use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_ledger::ledger::base_ledger::AnoncredsLedgerRead;
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use chrono::Utc;
use did_parser_nom::Did;
use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::{
        cred_issuance::{
            v1::{
                ack::{AckCredentialV1, AckCredentialV1Content},
                issue_credential::IssueCredentialV1,
                offer_credential::OfferCredentialV1,
                propose_credential::ProposeCredentialV1,
                request_credential::RequestCredentialV1,
                CredentialIssuanceV1,
            },
            CredentialIssuance,
        },
        notification::ack::{AckContent, AckDecorators, AckStatus},
        report_problem::ProblemReport,
        revocation::revoke::Revoke,
    },
    AriesMessage,
};
use uuid::Uuid;

use crate::{
    common::credentials::get_cred_rev_id,
    errors::error::prelude::*,
    handlers::revocation_notification::receiver::RevocationNotificationReceiver,
    protocols::issuance::holder::state_machine::{HolderFullState, HolderSM, HolderState},
};

fn build_credential_ack(thread_id: &str) -> AckCredentialV1 {
    let content = AckCredentialV1Content::builder()
        .inner(AckContent::builder().status(AckStatus::Ok).build())
        .build();
    let decorators = AckDecorators::builder()
        .thread(Thread::builder().thid(thread_id.to_owned()).build())
        .timing(Timing::builder().out_time(Utc::now()).build())
        .build();

    AckCredentialV1::builder()
        .id(Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorators)
        .build()
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

    pub fn get_proposal(&self) -> VcxResult<ProposeCredentialV1> {
        self.holder_sm.get_proposal()
    }

    pub fn create_with_proposal(
        source_id: &str,
        propose_credential: ProposeCredentialV1,
    ) -> VcxResult<Holder> {
        trace!(
            "Holder::create_with_proposal >>> source_id: {:?}, propose_credential: {:?}",
            source_id,
            propose_credential
        );
        let holder_sm = HolderSM::with_proposal(propose_credential, source_id.to_string());
        Ok(Holder { holder_sm })
    }

    pub fn create_from_offer(
        source_id: &str,
        credential_offer: OfferCredentialV1,
    ) -> VcxResult<Holder> {
        trace!(
            "Holder::create_from_offer >>> source_id: {:?}, credential_offer: {:?}",
            source_id,
            credential_offer
        );
        let holder_sm = HolderSM::from_offer(credential_offer, source_id.to_string());
        Ok(Holder { holder_sm })
    }

    pub fn set_proposal(&mut self, credential_proposal: ProposeCredentialV1) -> VcxResult<()> {
        self.holder_sm = self.holder_sm.clone().set_proposal(credential_proposal)?;
        Ok(())
    }

    pub async fn prepare_credential_request(
        &mut self,
        wallet: &impl BaseWallet,
        ledger: &impl AnoncredsLedgerRead,
        anoncreds: &impl BaseAnonCreds,
        my_pw_did: Did,
    ) -> VcxResult<AriesMessage> {
        self.holder_sm = self
            .holder_sm
            .clone()
            .prepare_credential_request(wallet, ledger, anoncreds, my_pw_did)
            .await?;
        match self.get_state() {
            HolderState::Failed => Ok(self.get_problem_report()?.into()),
            HolderState::RequestSet => Ok(self.get_msg_credential_request()?.into()),
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Holder::prepare_credential_request >> reached unexpected state after calling \
                 prepare_credential_request",
            )),
        }
    }

    pub fn get_msg_credential_request(&self) -> VcxResult<RequestCredentialV1> {
        match self.holder_sm.state {
            HolderFullState::RequestSet(ref state) => {
                let mut msg: RequestCredentialV1 = state.msg_credential_request.clone();
                let timing = Timing::builder().out_time(Utc::now()).build();
                msg.decorators.timing = Some(timing);
                Ok(msg)
            }
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotReady,
                "Invalid action",
            )),
        }
    }

    pub fn decline_offer<'a>(&'a mut self, comment: Option<&'a str>) -> VcxResult<ProblemReport> {
        self.holder_sm = self
            .holder_sm
            .clone()
            .decline_offer(comment.map(String::from))?;
        self.get_problem_report()
    }

    pub async fn process_credential(
        &mut self,
        wallet: &impl BaseWallet,
        ledger: &impl AnoncredsLedgerRead,
        anoncreds: &impl BaseAnonCreds,
        credential: IssueCredentialV1,
    ) -> VcxResult<()> {
        self.holder_sm = self
            .holder_sm
            .clone()
            .receive_credential(wallet, ledger, anoncreds, credential)
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

    pub fn get_offer(&self) -> VcxResult<OfferCredentialV1> {
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

    pub async fn is_revokable(&self, ledger: &impl AnoncredsLedgerRead) -> VcxResult<bool> {
        self.holder_sm.is_revokable(ledger).await
    }

    pub async fn is_revoked(
        &self,
        wallet: &impl BaseWallet,
        ledger: &impl AnoncredsLedgerRead,
        anoncreds: &impl BaseAnonCreds,
    ) -> VcxResult<bool> {
        self.holder_sm.is_revoked(wallet, ledger, anoncreds).await
    }

    pub async fn delete_credential(
        &self,
        wallet: &impl BaseWallet,
        anoncreds: &impl BaseAnonCreds,
    ) -> VcxResult<()> {
        self.holder_sm.delete_credential(wallet, anoncreds).await
    }

    pub fn get_credential_status(&self) -> VcxResult<u32> {
        Ok(self.holder_sm.credential_status())
    }

    pub async fn get_cred_rev_id(
        &self,
        wallet: &impl BaseWallet,
        anoncreds: &impl BaseAnonCreds,
    ) -> VcxResult<u32> {
        get_cred_rev_id(wallet, anoncreds, &self.get_cred_id()?).await
    }

    pub async fn handle_revocation_notification(
        &self,
        ledger: &impl AnoncredsLedgerRead,
        anoncreds: &impl BaseAnonCreds,
        wallet: &impl BaseWallet,
        notification: Revoke,
    ) -> VcxResult<()> {
        if self.holder_sm.is_revokable(ledger).await? {
            // TODO: Store to remember notification was received along with details
            RevocationNotificationReceiver::build(
                self.get_rev_reg_id()?,
                self.get_cred_rev_id(wallet, anoncreds).await?,
            )
            .handle_revocation_notification(notification)
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

    pub async fn process_aries_msg(
        &mut self,
        wallet: &impl BaseWallet,
        ledger: &impl AnoncredsLedgerRead,
        anoncreds: &impl BaseAnonCreds,
        message: AriesMessage,
    ) -> VcxResult<()> {
        let holder_sm = match message {
            AriesMessage::CredentialIssuance(CredentialIssuance::V1(
                CredentialIssuanceV1::OfferCredential(offer),
            )) => self.holder_sm.clone().receive_offer(offer)?,
            AriesMessage::CredentialIssuance(CredentialIssuance::V1(
                CredentialIssuanceV1::IssueCredential(credential),
            )) => {
                self.holder_sm
                    .clone()
                    .receive_credential(wallet, ledger, anoncreds, credential)
                    .await?
            }
            // TODO: What about credential issuance problem report?
            AriesMessage::ReportProblem(report) => {
                self.holder_sm.clone().receive_problem_report(report)?
            }
            _ => self.holder_sm.clone(),
        };
        self.holder_sm = holder_sm;
        Ok(())
    }

    pub fn get_final_message(&self) -> VcxResult<Option<AriesMessage>> {
        match &self.holder_sm.state {
            HolderFullState::Finished(state) if Some(true) == state.ack_requested => {
                let ack_msg = build_credential_ack(&self.get_thread_id()?);
                Ok(Some(ack_msg.into()))
            }
            _ => Ok(None),
        }
    }
}
