use std::sync::Arc;

use messages2::msg_fields::protocols::cred_issuance::propose_credential::ProposeCredential;

use crate::core::profile::profile::Profile;
use crate::errors::error::prelude::*;
use crate::protocols::issuance::is_cred_def_revokable;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProposalSentState {
    pub credential_proposal: ProposeCredential,
}

impl ProposalSentState {
    pub fn new(credential_proposal: ProposeCredential) -> Self {
        Self { credential_proposal }
    }

    pub async fn is_revokable(&self, profile: &Arc<dyn Profile>) -> VcxResult<bool> {
        is_cred_def_revokable(profile, &self.credential_proposal.content.cred_def_id).await
    }
}
