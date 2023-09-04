use crate::handlers::util::OfferInfo;
use messages::msg_fields::protocols::cred_issuance::propose_credential::ProposeCredential;

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
