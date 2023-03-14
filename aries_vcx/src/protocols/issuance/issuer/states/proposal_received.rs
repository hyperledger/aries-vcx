use messages::protocols::issuance::{
    credential_offer::{CredentialOffer, OfferInfo},
    credential_proposal::CredentialProposal,
};

use crate::protocols::issuance::issuer::states::offer_sent::OfferSentState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProposalReceivedState {
    pub credential_proposal: CredentialProposal,
    pub offer_info: Option<OfferInfo>,
}

impl ProposalReceivedState {
    pub fn new(credential_proposal: CredentialProposal, offer_info: Option<OfferInfo>) -> Self {
        Self {
            credential_proposal,
            offer_info,
        }
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
