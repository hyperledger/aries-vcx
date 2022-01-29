use crate::error::prelude::*;
use crate::handlers::issuance::is_cred_def_revokable;
use crate::messages::issuance::credential_proposal::CredentialProposal;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProposalSentState {
    pub credential_proposal: CredentialProposal,
}

impl ProposalSentState {
    pub fn new(credential_proposal: CredentialProposal) -> Self {
        Self { credential_proposal }
    }

    pub fn is_revokable(&self) -> VcxResult<bool> {
        is_cred_def_revokable(&self.credential_proposal.cred_def_id)
    }
}
