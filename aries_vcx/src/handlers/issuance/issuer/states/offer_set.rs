use crate::handlers::issuance::issuer::states::finished::FinishedState;
use crate::handlers::issuance::issuer::states::offer_sent::OfferSentState;
use crate::messages::a2a::MessageId;
use crate::messages::status::Status;
use crate::handlers::issuance::issuer::states::OfferInfo;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OfferSetState {
    pub offer_info: OfferInfo,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
}

impl OfferSetState {
    pub fn new(cred_def_id: &str, credential_json: &str, rev_reg_id: Option<String>, tails_file: Option<String>) -> Self {
        OfferSetState {
            offer_info: OfferInfo {
                cred_def_id: cred_def_id.to_string(),
                credential_json: credential_json.to_string(),
            },
            rev_reg_id,
            tails_file,
        }
    }
}

impl From<OfferSetState> for FinishedState {
    fn from(_state: OfferSetState) -> Self {
        trace!("SM is now in Finished state");
        FinishedState {
            cred_id: None,
            thread_id: String::new(),
            revocation_info_v1: None,
            status: Status::Undefined,
        }
    }
}

impl From<(OfferSetState, String, MessageId)> for OfferSentState {
    fn from((state, offer, sent_id): (OfferSetState, String, MessageId)) -> Self {
        trace!("SM is now in OfferSent state");
        OfferSentState {
            offer,
            cred_data: state.offer_info.credential_json,
            rev_reg_id: state.rev_reg_id,
            tails_file: state.tails_file,
            thread_id: sent_id.0,
        }
    }
}
