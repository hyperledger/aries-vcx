use std::sync::Arc;

use crate::core::profile::profile::Profile;
use crate::error::prelude::*;
use messages::issuance::credential_proposal::CredentialProposal;
use crate::protocols::issuance::is_cred_def_revokable;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProposalSentState {
    pub credential_proposal: CredentialProposal,
}

impl ProposalSentState {
    pub fn new(credential_proposal: CredentialProposal) -> Self {
        Self { credential_proposal }
    }

    pub async fn is_revokable(&self, profile: &Arc<dyn Profile>) -> VcxResult<bool> {
        is_cred_def_revokable(profile, &self.credential_proposal.cred_def_id).await
    }
}
