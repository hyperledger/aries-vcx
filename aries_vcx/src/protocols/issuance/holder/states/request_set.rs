use messages::msg_fields::protocols::cred_issuance::issue_credential::IssueCredential;
use messages::msg_fields::protocols::cred_issuance::request_credential::RequestCredential;

use crate::errors::error::prelude::*;
use crate::handlers::util::Status;
use crate::protocols::issuance::holder::states::finished::FinishedHolderState;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestSetState {
    pub req_meta: String,
    pub cred_def_json: String,
    pub msg_credential_request: RequestCredential,
}

impl From<(RequestSetState, String, IssueCredential, Option<String>)> for FinishedHolderState {
    fn from(
        (_, cred_id, credential, rev_reg_def_json): (RequestSetState, String, IssueCredential, Option<String>),
    ) -> Self {
        let ack_requested = credential.decorators.please_ack.is_some();
        FinishedHolderState {
            cred_id: Some(cred_id),
            credential: Some(credential),
            status: Status::Success,
            rev_reg_def_json,
            ack_requested: Some(ack_requested),
        }
    }
}

impl RequestSetState {
    pub fn is_revokable(&self) -> VcxResult<bool> {
        let parsed_cred_def: serde_json::Value = serde_json::from_str(&self.cred_def_json).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::SerializationError,
                format!(
                    "Failed deserialize credential definition json {}\nError: {}",
                    self.cred_def_json, err
                ),
            )
        })?;
        Ok(!parsed_cred_def["value"]["revocation"].is_null())
    }
}
