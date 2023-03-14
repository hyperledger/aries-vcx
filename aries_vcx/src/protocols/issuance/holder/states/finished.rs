use messages::{
    concepts::problem_report::ProblemReport,
    protocols::issuance::credential::{Credential, CredentialData},
    status::Status,
};

use crate::errors::error::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FinishedHolderState {
    pub cred_id: Option<String>,
    pub credential: Option<Credential>,
    pub status: Status,
    pub rev_reg_def_json: Option<String>,
}

impl FinishedHolderState {
    pub fn get_attributes(&self) -> VcxResult<String> {
        let attach = self.get_attachment()?;
        let cred_data: CredentialData = serde_json::from_str(&attach).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Cannot deserialize {:?}, into CredentialData, err: {:?}", attach, err),
            )
        })?;

        let mut new_map = serde_json::map::Map::new();
        match cred_data.values.as_object() {
            Some(values) => {
                for (key, value) in values {
                    let val = value["raw"]
                        .as_str()
                        .ok_or(AriesVcxError::from_msg(
                            AriesVcxErrorKind::InvalidJson,
                            "Missing raw encoding on credential value",
                        ))?
                        .into();
                    new_map.insert(key.clone(), val);
                }
                Ok(serde_json::Value::Object(new_map).to_string())
            }
            _ => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Cannot convert {:?} into object", attach),
            )),
        }
    }

    pub fn get_attachment(&self) -> VcxResult<String> {
        let credential = self.credential.as_ref().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            "No credential found",
        ))?;
        credential.credentials_attach.content().map_err(|err| err.into())
    }

    // TODO: Avoid duplication
    pub fn get_tails_location(&self) -> VcxResult<String> {
        debug!("get_tails_location >>>");
        let rev_reg_def_json = self.rev_reg_def_json.as_ref().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            "No revocation registry definition found - is this credential revokable?",
        ))?;
        let rev_reg_def: serde_json::Value = serde_json::from_str(rev_reg_def_json).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::SerializationError,
                format!("Cannot deserialize {:?} into Value, err: {:?}", rev_reg_def_json, err),
            )
        })?;
        let value = rev_reg_def["value"].as_object().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!(
                "The field 'value' not found on rev_reg_def_json: {:?}",
                rev_reg_def_json
            ),
        ))?;
        let tails_location = value["tailsLocation"].as_str().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!(
                "The field 'tailsLocation' not found on rev_reg_def_json: {:?}",
                self.rev_reg_def_json
            ),
        ))?;
        trace!("get_tails_location <<< tails_location: {}", tails_location.to_string());
        Ok(tails_location.to_string())
    }

    pub fn get_tails_hash(&self) -> VcxResult<String> {
        let rev_reg_def_json = self.rev_reg_def_json.as_ref().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            "No revocation registry definition found - is this credential revokable?",
        ))?;
        let rev_reg_def: serde_json::Value = serde_json::from_str(rev_reg_def_json).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::SerializationError,
                format!("Cannot deserialize {:?} into Value, err: {:?}", rev_reg_def_json, err),
            )
        })?;
        let value = rev_reg_def["value"].as_object().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!(
                "The field 'value' not found on rev_reg_def_json: {:?}",
                rev_reg_def_json
            ),
        ))?;
        let tails_hash = value["tailsHash"].as_str().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!(
                "The field 'tailsLocation' not found on rev_reg_def_json: {:?}",
                self.rev_reg_def_json
            ),
        ))?;
        Ok(tails_hash.to_string())
    }

    pub fn get_rev_reg_id(&self) -> VcxResult<String> {
        let rev_reg_def_json = self.rev_reg_def_json.as_ref().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            "No revocation registry definition found - is this credential revokable?",
        ))?;
        let rev_reg_def: serde_json::Value = serde_json::from_str(rev_reg_def_json).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::SerializationError,
                format!("Cannot deserialize {:?} into Value, err: {:?}", rev_reg_def_json, err),
            )
        })?;
        let rev_reg_def_id = rev_reg_def["id"].as_str().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!("The field 'id' not found on rev_reg_def_json: {:?}", rev_reg_def_json),
        ))?;
        Ok(rev_reg_def_id.to_string())
    }

    pub fn get_cred_id(&self) -> VcxResult<String> {
        self.cred_id.clone().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            "The field 'cred_id' not found on FinishedHolderState".to_string(),
        ))
    }

    pub fn is_revokable(&self) -> VcxResult<bool> {
        Ok(self.rev_reg_def_json.is_some())
    }
}

impl From<ProblemReport> for FinishedHolderState {
    fn from(problem_report: ProblemReport) -> Self {
        trace!("SM is now in Finished state");
        FinishedHolderState {
            cred_id: None,
            credential: None,
            status: Status::Failed(problem_report),
            rev_reg_def_json: None,
        }
    }
}
