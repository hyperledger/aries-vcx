use serde_json;
use serde_json::Value;

use api::VcxStateType;
use error::prelude::*;
use indyvc::proofs::proof_request::NonRevokedInterval;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct CredInfoVerifier {
    pub schema_id: String,
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub timestamp: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct CredInfoProver {
    pub requested_attr: String,
    pub referent: String,
    pub schema_id: String,
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub cred_rev_id: Option<String>,
    pub revocation_interval: Option<NonRevokedInterval>,
    pub tails_file: Option<String>,
    pub timestamp: Option<u64>,
}

pub fn get_credential_info(proof: &str) -> VcxResult<Vec<CredInfoVerifier>> {
    let mut rtn = Vec::new();

    let credentials: Value = serde_json::from_str(&proof)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize libndy proof: {}", err)))?;

    if let Value::Array(ref identifiers) = credentials["identifiers"] {
        for identifier in identifiers {
            if let (Some(schema_id), Some(cred_def_id)) = (identifier["schema_id"].as_str(),
                                                           identifier["cred_def_id"].as_str()) {
                let rev_reg_id = identifier["rev_reg_id"]
                    .as_str()
                    .map(|x| x.to_string());

                let timestamp = identifier["timestamp"].as_u64();
                rtn.push(
                    CredInfoVerifier {
                        schema_id: schema_id.to_string(),
                        cred_def_id: cred_def_id.to_string(),
                        rev_reg_id,
                        timestamp,
                    }
                );
            } else { return Err(VcxError::from_msg(VcxErrorKind::InvalidProofCredentialData, "Cannot get identifiers")); }
        }
    }

    Ok(rtn)
}
