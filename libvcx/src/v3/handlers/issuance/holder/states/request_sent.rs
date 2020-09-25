use v3::handlers::issuance::holder::states::finished::FinishedHolderState;
use v3::messages::issuance::credential::Credential;
use v3::messages::error::ProblemReport;
use v3::messages::status::Status;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestSentState {
    pub req_meta: String,
    pub cred_def_json: String,
    pub connection_handle: u32,
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
