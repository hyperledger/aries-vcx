use messages::protocols::issuance::credential_offer::{CredentialOffer, OfferInfo};

use crate::protocols::issuance::issuer::states::offer_sent::OfferSentState;

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct InitialIssuerState {}

impl From<(OfferInfo, CredentialOffer)> for OfferSentState {
    fn from((offer_info, offer): (OfferInfo, CredentialOffer)) -> Self {
        trace!("SM is now in OfferSent state");
        OfferSentState {
            offer,
            cred_data: offer_info.credential_json,
            rev_reg_id: offer_info.rev_reg_id,
            tails_file: offer_info.tails_file,
        }
    }
}
