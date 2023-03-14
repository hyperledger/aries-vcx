use messages::protocols::issuance::credential_offer::CredentialOffer;

use crate::protocols::issuance::issuer::states::offer_sent::OfferSentState;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OfferSetState {
    pub offer: CredentialOffer,
    pub credential_json: String,
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
}

impl OfferSetState {
    pub fn new(
        cred_offer_msg: CredentialOffer,
        credential_json: &str,
        cred_def_id: &str,
        rev_reg_id: Option<String>,
        tails_file: Option<String>,
    ) -> Self {
        OfferSetState {
            offer: cred_offer_msg,
            credential_json: credential_json.into(),
            cred_def_id: cred_def_id.into(),
            rev_reg_id,
            tails_file,
        }
    }
}

impl From<OfferSetState> for OfferSentState {
    fn from(state: OfferSetState) -> Self {
        trace!("SM is now in OfferSent state");
        OfferSentState {
            offer: state.offer,
            cred_data: state.credential_json,
            rev_reg_id: state.rev_reg_id,
            tails_file: state.tails_file,
        }
    }
}
