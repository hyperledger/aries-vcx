use anoncreds_types::data_types::identifiers::schema_id::SchemaId;
use messages::msg_fields::protocols::cred_issuance::v1::{
    issue_credential::IssueCredentialV1, request_credential::RequestCredentialV1,
};

use crate::{
    errors::error::prelude::*, handlers::util::Status,
    protocols::issuance::holder::states::finished::FinishedHolderState,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestSetState {
    pub req_meta: String,
    pub cred_def_json: String,
    pub schema_id: SchemaId,
    pub msg_credential_request: RequestCredentialV1,
}

impl From<(RequestSetState, String, IssueCredentialV1, Option<String>)> for FinishedHolderState {
    fn from(
        (_, cred_id, credential, rev_reg_def_json): (
            RequestSetState,
            String,
            IssueCredentialV1,
            Option<String>,
        ),
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
        let parsed_cred_def: serde_json::Value = serde_json::from_str(&self.cred_def_json)
            .map_err(|err| {
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
