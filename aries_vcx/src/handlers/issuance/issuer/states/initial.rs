use crate::handlers::issuance::issuer::states::offer_sent::OfferSentState;
use crate::messages::issuance::credential_offer::OfferInfo;
use crate::messages::a2a::MessageId;

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct InitialIssuerState {}

impl From<(OfferInfo, String, MessageId)> for OfferSentState {
    fn from((offer_info, offer, sent_id): (OfferInfo, String, MessageId)) -> Self {
        trace!("SM is now in OfferSent state");
        OfferSentState {
            offer,
            cred_data: offer_info.credential_json,
            rev_reg_id: offer_info.rev_reg_id,
            tails_file: offer_info.tails_file,
            thread_id: sent_id.0,
        }
    }
}
