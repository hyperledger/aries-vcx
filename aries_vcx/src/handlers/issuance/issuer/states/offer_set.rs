use crate::handlers::issuance::issuer::states::finished::FinishedState;
use crate::messages::status::Status;
use crate::messages::issuance::credential_offer::OfferInfo;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OfferSetState {
    pub offer_info: OfferInfo,
}

impl OfferSetState {
    pub fn new(cred_def_id: &str, credential_json: &str, rev_reg_id: Option<String>, tails_file: Option<String>) -> Self {
        OfferSetState {
            offer_info: OfferInfo {
                cred_def_id: cred_def_id.to_string(),
                credential_json: credential_json.to_string(),
                rev_reg_id,
                tails_file
            }
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
