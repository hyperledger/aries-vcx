use std::sync::Arc;

use messages::protocols::issuance::credential_proposal::CredentialProposal;

use crate::{core::profile::profile::Profile, errors::error::prelude::*, protocols::issuance::is_cred_def_revokable};

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
