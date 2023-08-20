use crate::{handlers::util::OfferInfo, protocols::issuance::issuer::states::offer_set::OfferSetState};
use messages::msg_fields::protocols::cred_issuance::{
    offer_credential::OfferCredential, propose_credential::ProposeCredential,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProposalReceivedState {
    pub credential_proposal: ProposeCredential,
    pub offer_info: Option<OfferInfo>,
}

impl ProposalReceivedState {
    pub fn new(credential_proposal: ProposeCredential, offer_info: Option<OfferInfo>) -> Self {
        Self {
            credential_proposal,
            offer_info,
        }
    }
}

impl From<(OfferCredential, OfferInfo)> for OfferSetState {
    fn from((offer, offer_info): (OfferCredential, OfferInfo)) -> Self {
        trace!("SM is now in OfferSet state");
        OfferSetState {
            offer,
            credential_json: offer_info.credential_json,
            cred_def_id: offer_info.cred_def_id,
            rev_reg_id: offer_info.rev_reg_id,
            tails_file: offer_info.tails_file,
        }
    }
}
