use crate::aries::handlers::issuance::holder::states::finished::FinishedHolderState;
use crate::aries::messages::issuance::credential::Credential;
use crate::aries::messages::error::ProblemReport;
use crate::aries::messages::status::Status;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestSentState {
    pub req_meta: String,
    pub cred_def_json: String
}

impl From<(RequestSentState, String, Credential, Option<String>)> for FinishedHolderState {
    fn from((_, cred_id, credential, rev_reg_def_json): (RequestSentState, String, Credential, Option<String>)) -> Self {
        trace!("SM is now in Finished state");
        FinishedHolderState {
            cred_id: Some(cred_id),
            credential: Some(credential),
            status: Status::Success,
            rev_reg_def_json,
        }
    }
}

impl From<(RequestSentState, ProblemReport)> for FinishedHolderState {
    fn from((_, problem_report): (RequestSentState, ProblemReport)) -> Self {
        trace!("SM is now in Finished state");
        FinishedHolderState {
            cred_id: None,
            credential: None,
            status: Status::Failed(problem_report),
            rev_reg_def_json: None,
        }
    }
}
