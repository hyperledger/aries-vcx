use aries_vcx::{
    aries_vcx_core::{
        anoncreds::base_anoncreds::BaseAnonCreds, ledger::base_ledger::AnoncredsLedgerRead,
    },
    handlers::issuance::holder::Holder as VcxHolder,
    messages::{
        msg_fields::protocols::{
            cred_issuance::v1::{
                issue_credential::IssueCredentialV1, offer_credential::OfferCredentialV1,
                propose_credential::ProposeCredentialV1, request_credential::RequestCredentialV1,
            },
            report_problem::ProblemReport,
        },
        AriesMessage,
    },
    protocols::issuance::holder::state_machine::HolderState,
};
use std::sync::{Arc, Mutex};

use crate::{errors::error::VcxUniFFIResult, runtime::block_on};
pub struct Holder {
    handler: Mutex<VcxHolder>,
}

// initializers

pub fn create(source_id: String) -> VcxUniFFIResult<Arc<Holder>> {
    block_on(async {
        let handler = Mutex::new(VcxHolder::create(&source_id)?);
        Ok(Arc::new(Holder { handler }))
    })
}

pub fn create_from_offer(
    source_id: String,
    cred: OfferCredentialV1,
) -> VcxUniFFIResult<Arc<Holder>> {
    block_on(async {
        let handler = Mutex::new(VcxHolder::create_from_offer(&source_id, cred)?);
        Ok(Arc::new(Holder { handler }))
    })
}

pub fn create_with_proposal(
    source_id: String,
    propose_credential: ProposeCredentialV1,
) -> VcxUniFFIResult<Arc<Holder>> {
    block_on(async {
        let handler = Mutex::new(VcxHolder::create_with_proposal(
            &source_id,
            propose_credential,
        )?);
        Ok(Arc::new(Holder { handler }))
    })
}

impl Holder {
    pub fn set_proposal(
        &mut self,
        credential_proposal: ProposeCredentialV1,
    ) -> VcxUniFFIResult<()> {
        self.handler.lock()?.set_proposal(credential_proposal)?;
        Ok(())
    }

    pub fn prepare_credential_request(
        &mut self,
        ledger: &impl AnoncredsLedgerRead,
        anoncreds: &impl BaseAnonCreds,
        my_pw_did: String,
    ) -> VcxUniFFIResult<()> {
        block_on(async {
            self.handler
                .lock()?
                .prepare_credential_request(ledger, anoncreds, my_pw_did)
                .await?;
            Ok(())
        })
    }

    pub fn get_msg_credential_request(&self) -> VcxUniFFIResult<RequestCredentialV1> {
        Ok(self.handler.lock()?.clone().get_msg_credential_request()?)
    }

    pub fn decline_offer(&self) -> VcxUniFFIResult<ProblemReport> {
        Ok(self
            .handler
            .lock()?
            .clone()
            .decline_offer(Some(&String::new()))?)
    }

    pub async fn process_credential(
        &mut self,
        ledger: &impl AnoncredsLedgerRead,
        anoncreds: &impl BaseAnonCreds,
        credential: IssueCredentialV1,
    ) -> VcxUniFFIResult<()> {
        block_on(async {
            self.handler
                .lock()?
                .process_credential(ledger, anoncreds, credential)
                .await?;
            Ok(())
        })
    }

    pub fn is_terminal_state(&self) -> VcxUniFFIResult<bool> {
        Ok(self.handler.lock()?.is_terminal_state())
    }

    pub fn get_state(&self) -> VcxUniFFIResult<HolderState> {
        Ok(self.handler.lock()?.get_state())
    }

    pub fn get_source_id(&self) -> VcxUniFFIResult<String> {
        Ok(self.handler.lock()?.get_source_id())
    }

    pub fn get_credential(&self) -> VcxUniFFIResult<(String, AriesMessage)> {
        Ok(self.handler.lock()?.get_credential()?)
    }

    pub fn get_attributes(&self) -> VcxUniFFIResult<String> {
        Ok(self.handler.lock()?.get_attributes()?)
    }

    pub fn get_attachment(&self) -> VcxUniFFIResult<String> {
        Ok(self.handler.lock()?.get_attachment()?)
    }

    pub fn get_offer(&self) -> VcxUniFFIResult<OfferCredentialV1> {
        Ok(self.handler.lock()?.get_offer()?)
    }

    pub fn get_tails_location(&self) -> VcxUniFFIResult<String> {
        Ok(self.handler.lock()?.get_tails_location()?)
    }

    pub fn get_tails_hash(&self) -> VcxUniFFIResult<String> {
        Ok(self.handler.lock()?.get_tails_hash()?)
    }

    pub fn get_rev_reg_id(&self) -> VcxUniFFIResult<String> {
        Ok(self.handler.lock()?.get_rev_reg_id()?)
    }

    pub fn get_cred_id(&self) -> VcxUniFFIResult<String> {
        Ok(self.handler.lock()?.get_cred_id()?)
    }

    pub fn get_thread_id(&self) -> VcxUniFFIResult<String> {
        Ok(self.handler.lock()?.get_thread_id()?)
    }

    pub async fn is_revokable(&self, ledger: &impl AnoncredsLedgerRead) -> VcxUniFFIResult<bool> {
        block_on(async { Ok(self.handler.lock()?.is_revokable(ledger).await?) })
    }

    pub async fn is_revoked(
        &self,
        ledger: &impl AnoncredsLedgerRead,
        anoncreds: &impl BaseAnonCreds,
    ) -> VcxUniFFIResult<bool> {
        Ok(self.handler.lock()?.is_revoked(ledger, anoncreds).await?)
    }

    pub fn get_cred_rev_id(&self, anoncreds: &impl BaseAnonCreds) -> VcxUniFFIResult<String> {
        block_on(async { Ok(self.handler.lock()?.get_cred_rev_id(anoncreds).await?) })
    }

    pub fn get_problem_report(&self) -> VcxUniFFIResult<ProblemReport> {
        Ok(self.handler.lock()?.get_problem_report()?)
    }

    pub fn get_final_message(&self) -> VcxUniFFIResult<Option<AriesMessage>> {
        Ok(self.handler.lock()?.get_final_message()?)
    }
}
