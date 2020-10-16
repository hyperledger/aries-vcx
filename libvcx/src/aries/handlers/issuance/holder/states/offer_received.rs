use std::collections::HashMap;
use error::prelude::*;
use aries::handlers::issuance::holder::states::finished::FinishedHolderState;
use aries::handlers::issuance::holder::states::request_sent::RequestSentState;
use aries::messages::error::ProblemReport;
use aries::messages::issuance::credential_offer::CredentialOffer;
use aries::messages::status::Status;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OfferReceivedState {
    pub offer: CredentialOffer
}

impl From<(OfferReceivedState, String, String, u32)> for RequestSentState {
    fn from((_state, req_meta, cred_def_json, connection_handle): (OfferReceivedState, String, String, u32)) -> Self {
        trace!("SM is now in RequestSent state");
        RequestSentState {
            req_meta,
            cred_def_json,
            connection_handle,
        }
    }
}

impl From<(OfferReceivedState, ProblemReport)> for FinishedHolderState {
    fn from((_state, problem_report): (OfferReceivedState, ProblemReport)) -> Self {
        trace!("SM is now in Finished state");
        FinishedHolderState {
            cred_id: None,
            credential: None,
            status: Status::Failed(problem_report),
            rev_reg_def_json: None,
        }
    }
}

impl OfferReceivedState {
    pub fn new(offer: CredentialOffer) -> Self {
        OfferReceivedState {
            offer,
        }
    }

    pub fn get_attributes(&self) -> VcxResult<String> {
        let mut new_map: HashMap<String, String> = HashMap::new();
        self.offer.credential_preview.attributes.iter().for_each(|attribute| {
            new_map.insert(attribute.name.clone(), attribute.value.clone());
        });
        serde_json::to_string(&new_map)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Failed to serialize {:?}, err: {}", new_map, err)))
    }
}
