use vdrtools_sys::{PoolHandle, WalletHandle};

use crate::error::prelude::*;
use crate::protocols::issuance::is_cred_def_revokable;
use messages::issuance::credential_proposal::CredentialProposal;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProposalSentState {
    pub credential_proposal: CredentialProposal,
}

impl ProposalSentState {
    pub fn new(credential_proposal: CredentialProposal) -> Self {
        Self { credential_proposal }
    }

    pub async fn is_revokable(&self, wallet_handle: WalletHandle, pool_handle: PoolHandle) -> VcxResult<bool> {
        is_cred_def_revokable(wallet_handle, pool_handle, &self.credential_proposal.cred_def_id).await
    }
}
