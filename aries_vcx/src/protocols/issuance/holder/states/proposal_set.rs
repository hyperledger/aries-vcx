use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use std::sync::Arc;

use messages::msg_fields::protocols::cred_issuance::propose_credential::ProposeCredential;

use crate::errors::error::prelude::*;
use crate::protocols::issuance::is_cred_def_revokable;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProposalSetState {
    pub credential_proposal: ProposeCredential,
}

impl ProposalSetState {
    pub fn new(credential_proposal: ProposeCredential) -> Self {
        Self { credential_proposal }
    }

    pub async fn is_revokable(&self, ledger: &Arc<dyn AnoncredsLedgerRead>) -> VcxResult<bool> {
        is_cred_def_revokable(ledger, &self.credential_proposal.content.cred_def_id).await
    }
}
