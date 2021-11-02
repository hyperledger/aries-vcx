use crate::error::prelude::*;
use crate::messages::issuance::credential_proposal::CredentialProposal;
use crate::handlers::issuance::is_cred_def_revokable;
use crate::handlers::issuance::issuer::states::offer_sent::OfferSentState;
use crate::messages::issuance::credential_offer::OfferInfo;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProposalReceivedState {
    pub credential_proposal: CredentialProposal,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
    pub offer_info: Option<OfferInfo>
}

impl ProposalReceivedState {
    pub fn new(credential_proposal: CredentialProposal, rev_reg_id: Option<String>, tails_file: Option<String>, offer_info: Option<OfferInfo>) -> Self {
        Self {
            credential_proposal,
            rev_reg_id,
            tails_file,
            offer_info
        }
    }

    pub fn is_revokable(&self) -> VcxResult<bool> {
        is_cred_def_revokable(&self.credential_proposal.cred_def_id)
    }
}

impl From<(String, String, String, Option<String>, Option<String>)> for OfferSentState {
    fn from((cred_data, offer, thread_id, rev_reg_id, tails_file): (String, String, String, Option<String>, Option<String>)) -> Self {
        trace!("SM is now in OfferSent state");
        OfferSentState {
            offer,
            cred_data,
            rev_reg_id,
            tails_file,
            thread_id,
        }
    }
}
