use messages::msg_fields::protocols::cred_issuance::offer_credential::OfferCredential;

use crate::{handlers::util::OfferInfo, protocols::issuance::issuer::states::offer_set::OfferSetState};

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct InitialIssuerState {}

impl From<(OfferInfo, OfferCredential)> for OfferSetState {
    fn from((offer_info, offer): (OfferInfo, OfferCredential)) -> Self {
        trace!("SM is now in OfferSent state");
        OfferSetState {
            offer,
            credential_json: offer_info.credential_json,
            cred_def_id: offer_info.cred_def_id,
            rev_reg_id: offer_info.rev_reg_id,
            tails_file: offer_info.tails_file,
        }
    }
}
