use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use messages::msg_fields::protocols::cred_issuance::v1::propose_credential::ProposeCredentialV1;

use crate::{errors::error::prelude::*, protocols::issuance::is_cred_def_revokable};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProposalSetState {
    pub credential_proposal: ProposeCredentialV1,
}

impl ProposalSetState {
    pub fn new(credential_proposal: ProposeCredentialV1) -> Self {
        Self {
            credential_proposal,
        }
    }

    pub async fn is_revokable(&self, ledger: &impl AnoncredsLedgerRead) -> VcxResult<bool> {
        is_cred_def_revokable(ledger, &self.credential_proposal.content.cred_def_id).await
    }
}
