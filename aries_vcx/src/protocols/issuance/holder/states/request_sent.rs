use messages::{protocols::issuance::credential::Credential, status::Status};

use crate::{errors::error::prelude::*, protocols::issuance::holder::states::finished::FinishedHolderState};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestSentState {
    pub req_meta: String,
    pub cred_def_json: String,
}

impl From<(RequestSentState, String, Credential, Option<String>)> for FinishedHolderState {
    fn from(
        (_, cred_id, credential, rev_reg_def_json): (RequestSentState, String, Credential, Option<String>),
    ) -> Self {
        trace!("SM is now in Finished state");
        FinishedHolderState {
            cred_id: Some(cred_id),
            credential: Some(credential),
            status: Status::Success,
            rev_reg_def_json,
        }
    }
}

impl RequestSentState {
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
