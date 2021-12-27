use crate::error::prelude::*;
use crate::messages::issuance::credential_proposal::CredentialProposal;
use crate::handlers::issuance::is_cred_def_revokable;
use crate::handlers::issuance::issuer::states::offer_sent::OfferSentState;
use crate::messages::issuance::credential_offer::{CredentialOffer, OfferInfo};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProposalReceivedState {
    pub credential_proposal: CredentialProposal,
    pub offer_info: Option<OfferInfo>
}

impl ProposalReceivedState {
    pub fn new(credential_proposal: CredentialProposal, offer_info: Option<OfferInfo>) -> Self {
        Self {
            credential_proposal,
            offer_info
        }
    }

    pub fn is_revokable(&self) -> VcxResult<bool> {
        is_cred_def_revokable(&self.credential_proposal.cred_def_id)
    }
}

impl From<(CredentialOffer, OfferInfo)> for OfferSentState {
    fn from((offer, offer_info): (CredentialOffer, OfferInfo)) -> Self {
        trace!("SM is now in OfferSent state");
        OfferSentState {
            offer,
            cred_data: offer_info.credential_json,
            rev_reg_id: offer_info.rev_reg_id,
            tails_file: offer_info.tails_file,
        }
    }
}
