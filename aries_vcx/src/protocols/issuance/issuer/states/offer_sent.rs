use messages::{
    concepts::problem_report::ProblemReport,
    protocols::issuance::{credential_offer::CredentialOffer, credential_request::CredentialRequest},
    status::Status,
};

use crate::protocols::issuance::issuer::{
    state_machine::RevocationInfoV1,
    states::{finished::FinishedState, requested_received::RequestReceivedState},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OfferSentState {
    pub offer: CredentialOffer,
    pub cred_data: String,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
}

impl From<OfferSentState> for FinishedState {
    fn from(state: OfferSentState) -> Self {
        trace!("SM is now in Finished state");
        FinishedState {
            cred_id: None,
            revocation_info_v1: Some(RevocationInfoV1 {
                cred_rev_id: None,
                rev_reg_id: state.rev_reg_id,
                tails_file: state.tails_file,
            }),
            status: Status::Undefined,
        }
    }
}

impl From<(OfferSentState, CredentialRequest)> for RequestReceivedState {
    fn from((state, request): (OfferSentState, CredentialRequest)) -> Self {
        trace!("SM is now in Request Received state");
        RequestReceivedState {
            offer: state.offer,
            cred_data: state.cred_data,
            rev_reg_id: state.rev_reg_id,
            tails_file: state.tails_file,
            request,
        }
    }
}

impl From<(OfferSentState, ProblemReport)> for FinishedState {
    fn from((state, err): (OfferSentState, ProblemReport)) -> Self {
        trace!("SM is now in Finished state");
        FinishedState {
            cred_id: None,
            revocation_info_v1: Some(RevocationInfoV1 {
                cred_rev_id: None,
                rev_reg_id: state.rev_reg_id,
                tails_file: state.tails_file,
            }),
            status: Status::Failed(err),
        }
    }
}
