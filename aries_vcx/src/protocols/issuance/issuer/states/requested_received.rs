use messages::msg_fields::protocols::{
    cred_issuance::v1::{offer_credential::OfferCredential, request_credential::RequestCredential},
    report_problem::ProblemReport,
};

use crate::{
    handlers::util::Status,
    protocols::issuance::issuer::{
        state_machine::RevocationInfoV1, states::finished::FinishedState,
    },
};

// TODO: Use OfferInfo instead of ind. fields
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestReceivedState {
    pub offer: OfferCredential,
    pub cred_data: String,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
    pub request: RequestCredential,
}

impl FinishedState {
    pub fn from_request_and_error(state: RequestReceivedState, err: ProblemReport) -> Self {
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
