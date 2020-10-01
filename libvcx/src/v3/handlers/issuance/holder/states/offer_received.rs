use v3::handlers::issuance::holder::states::finished::FinishedHolderState;
use v3::handlers::issuance::holder::states::request_sent::RequestSentState;
use v3::messages::error::ProblemReport;
use v3::messages::issuance::credential_offer::CredentialOffer;
use v3::messages::status::Status;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OfferReceivedState {
    pub offer: CredentialOffer
}


impl OfferReceivedState {
    pub fn new(offer: CredentialOffer) -> Self {
        OfferReceivedState {
            offer,
        }
    }
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
