use crate::handlers::util::Status;
use crate::protocols::issuance::issuer::state_machine::RevocationInfoV1;
use crate::protocols::issuance::issuer::states::credential_sent::CredentialSentState;
use crate::protocols::issuance::issuer::states::finished::FinishedState;
use messages2::msg_fields::protocols::cred_issuance::offer_credential::OfferCredential;
use messages2::msg_fields::protocols::cred_issuance::request_credential::RequestCredential;
use messages2::msg_fields::protocols::report_problem::ProblemReport;

// TODO: Use OfferInfo instead of ind. fields
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestReceivedState {
    pub offer: OfferCredential,
    pub cred_data: String,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
    pub request: RequestCredential,
}

impl From<(RequestReceivedState, Option<String>)> for CredentialSentState {
    fn from((state, cred_rev_id): (RequestReceivedState, Option<String>)) -> Self {
        trace!("SM is now in CredentialSent state");
        CredentialSentState {
            revocation_info_v1: Some(RevocationInfoV1 {
                cred_rev_id,
                rev_reg_id: state.rev_reg_id,
                tails_file: state.tails_file,
            }),
        }
    }
}

impl From<(RequestReceivedState, Option<String>)> for FinishedState {
    fn from((state, cred_rev_id): (RequestReceivedState, Option<String>)) -> Self {
        trace!("SM is now in Finished state");
        FinishedState {
            cred_id: None,
            revocation_info_v1: Some(RevocationInfoV1 {
                cred_rev_id,
                rev_reg_id: state.rev_reg_id,
                tails_file: state.tails_file,
            }),
            status: Status::Success,
        }
    }
}

impl From<(RequestReceivedState, ProblemReport)> for FinishedState {
    fn from((state, err): (RequestReceivedState, ProblemReport)) -> Self {
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
