use std::collections::HashMap;
use error::prelude::*;
use aries::messages::issuance::credential::{Credential, CredentialData};
use aries::messages::status::Status;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FinishedHolderState {
    pub cred_id: Option<String>,
    pub credential: Option<Credential>,
    pub status: Status,
    pub rev_reg_def_json: Option<String>,
}

impl FinishedHolderState {
    pub fn get_attributes(&self) -> VcxResult<String> {
        let credential = self.credential.as_ref().ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "No credential found"))?;
        let content = credential.credentials_attach.content()?;
        let cred_data: CredentialData = serde_json::from_str(&content)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize {:?}, into CredentialData, err: {:?}", content, err)))?;

        let mut new_map = serde_json::map::Map::new();
        match cred_data.values.as_object() {
            Some(values) => {
                for (key, value) in values {
                    let val = value["raw"]
                        .as_str()
                        .ok_or(VcxError::from_msg(VcxErrorKind::InvalidJson, "Missing raw encoding on credential value"))?
                        .into();
                    new_map.insert(key.clone(), val);
                };
                Ok(serde_json::Value::Object(new_map).to_string())
            }
            _ => Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot convert {:?} into object", content)))
        }
    }
}
