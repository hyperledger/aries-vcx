use messages::msg_fields::protocols::cred_issuance::v1::propose_credential::ProposeCredentialV1;

use crate::handlers::util::OfferInfo;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProposalReceivedState {
    pub credential_proposal: ProposeCredentialV1,
    pub offer_info: Option<OfferInfo>,
}

impl ProposalReceivedState {
    pub fn new(credential_proposal: ProposeCredentialV1, offer_info: Option<OfferInfo>) -> Self {
        Self {
            credential_proposal,
            offer_info,
        }
    }
}
