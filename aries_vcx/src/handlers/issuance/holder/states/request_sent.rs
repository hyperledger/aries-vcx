use crate::error::prelude::*;
use crate::handlers::issuance::holder::states::finished::FinishedHolderState;
use crate::messages::error::ProblemReport;
use crate::messages::issuance::credential::Credential;
use crate::messages::status::Status;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestSentState {
    pub req_meta: String,
    pub cred_def_json: String,
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

impl RequestSentState {
    pub fn is_revokable(&self) -> VcxResult<bool> {
        let parsed_cred_def: serde_json::Value = serde_json::from_str(&self.cred_def_json)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed deserialize credential definition json {}\nError: {}", self.cred_def_json, err)))?;
        Ok(!parsed_cred_def["value"]["revocation"].is_null())
    }
}
