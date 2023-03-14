use std::sync::Arc;

use serde_json::{self, Value};

use crate::{core::profile::profile::Profile, errors::error::prelude::*, global::settings, utils::openssl::encode};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CredInfoVerifier {
    pub schema_id: String,
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub timestamp: Option<u64>,
}

pub fn get_credential_info(proof: &str) -> VcxResult<Vec<CredInfoVerifier>> {
    let mut rtn = Vec::new();

    let credentials: Value = serde_json::from_str(proof).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!("Cannot deserialize libndy proof: {}", err),
        )
    })?;

    if let Value::Array(ref identifiers) = credentials["identifiers"] {
        for identifier in identifiers {
            if let (Some(schema_id), Some(cred_def_id)) =
                (identifier["schema_id"].as_str(), identifier["cred_def_id"].as_str())
            {
                let rev_reg_id = identifier["rev_reg_id"].as_str().map(|x| x.to_string());

                let timestamp = identifier["timestamp"].as_u64();
                rtn.push(CredInfoVerifier {
                    schema_id: schema_id.to_string(),
                    cred_def_id: cred_def_id.to_string(),
                    rev_reg_id,
                    timestamp,
                });
            } else {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidProofCredentialData,
                    "Cannot get identifiers",
                ));
            }
        }
    }

    Ok(rtn)
}

pub fn validate_proof_revealed_attributes(proof_json: &str) -> VcxResult<()> {
    if settings::indy_mocks_enabled() {
        return Ok(());
    }

    let proof: Value = serde_json::from_str(proof_json).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!("Cannot deserialize libndy proof: {}", err),
        )
    })?;

    let revealed_attrs = match proof["requested_proof"]["revealed_attrs"].as_object() {
        Some(revealed_attrs) => revealed_attrs,
        None => return Ok(()),
    };

    for (attr1_referent, info) in revealed_attrs.iter() {
        let raw = info["raw"].as_str().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidProof,
            format!("Cannot get raw value for \"{}\" attribute", attr1_referent),
        ))?;
        let encoded_ = info["encoded"].as_str().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidProof,
            format!("Cannot get encoded value for \"{}\" attribute", attr1_referent),
        ))?;

        let expected_encoded = encode(raw)?;

        if expected_encoded != *encoded_ {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidProof,
                format!(
                    "Encoded values are different. Expected: {}. From Proof: {}",
                    expected_encoded, encoded_
                ),
            ));
        }
    }

    Ok(())
}

pub async fn build_cred_defs_json_verifier(
    profile: &Arc<dyn Profile>,
    credential_data: &[CredInfoVerifier],
) -> VcxResult<String> {
    debug!("building credential_def_json for proof validation");
    let ledger = Arc::clone(profile).inject_ledger();
    let mut credential_json = json!({});

    for cred_info in credential_data.iter() {
        if credential_json.get(&cred_info.cred_def_id).is_none() {
            let cred_def_id = &cred_info.cred_def_id;
            let credential_def = ledger.get_cred_def(cred_def_id, None).await?;

            let credential_def = serde_json::from_str(&credential_def).map_err(|err| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidProofCredentialData,
                    format!("Cannot deserialize credential definition: {}", err),
                )
            })?;

            credential_json[cred_def_id] = credential_def;
        }
    }

    Ok(credential_json.to_string())
}

pub async fn build_schemas_json_verifier(
    profile: &Arc<dyn Profile>,
    credential_data: &[CredInfoVerifier],
) -> VcxResult<String> {
    debug!("building schemas json for proof validation");

    let ledger = Arc::clone(profile).inject_ledger();
    let mut schemas_json = json!({});

    for cred_info in credential_data.iter() {
        if schemas_json.get(&cred_info.schema_id).is_none() {
            let schema_id = &cred_info.schema_id;
            let schema_json = ledger
                .get_schema(schema_id, None)
                .await
                .map_err(|err| err.map(AriesVcxErrorKind::InvalidSchema, "Cannot get schema"))?;
            let schema_val = serde_json::from_str(&schema_json).map_err(|err| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidSchema,
                    format!("Cannot deserialize schema: {}", err),
                )
            })?;
            schemas_json[schema_id] = schema_val;
        }
    }

    Ok(schemas_json.to_string())
}

pub async fn build_rev_reg_defs_json(
    profile: &Arc<dyn Profile>,
    credential_data: &[CredInfoVerifier],
) -> VcxResult<String> {
    debug!("building rev_reg_def_json for proof validation");

    let ledger = Arc::clone(profile).inject_ledger();
    let mut rev_reg_defs_json = json!({});

    for cred_info in credential_data.iter() {
        let rev_reg_id = cred_info.rev_reg_id.as_ref().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidRevocationDetails,
            format!("Missing rev_reg_id in the record {:?}", cred_info),
        ))?;

        if rev_reg_defs_json.get(rev_reg_id).is_none() {
            let json = ledger.get_rev_reg_def_json(rev_reg_id).await?;
            let rev_reg_def_json = serde_json::from_str(&json).or(Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Failed to deserialize as json rev_reg_def: {}", json),
            )))?;
            rev_reg_defs_json[rev_reg_id] = rev_reg_def_json;
        }
    }

    Ok(rev_reg_defs_json.to_string())
}

pub async fn build_rev_reg_json(profile: &Arc<dyn Profile>, credential_data: &[CredInfoVerifier]) -> VcxResult<String> {
    debug!("building rev_reg_json for proof validation");

    let ledger = Arc::clone(profile).inject_ledger();
    let mut rev_regs_json = json!({});

    for cred_info in credential_data.iter() {
        let rev_reg_id = cred_info.rev_reg_id.as_ref().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidRevocationDetails,
            format!("Missing rev_reg_id in the record {:?}", cred_info),
        ))?;

        let timestamp = cred_info.timestamp.as_ref().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidRevocationTimestamp,
            format!("Revocation timestamp is missing on record {:?}", cred_info),
        ))?;

        if rev_regs_json.get(rev_reg_id).is_none() {
            let (id, rev_reg_json, timestamp) = ledger.get_rev_reg(rev_reg_id, timestamp.to_owned()).await?;
            let rev_reg_json: Value = serde_json::from_str(&rev_reg_json).or(Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Failed to deserialize as json: {}", rev_reg_json),
            )))?;
            let rev_reg_json = json!({ timestamp.to_string(): rev_reg_json });
            rev_regs_json[id] = rev_reg_json;
        }
    }

    Ok(rev_regs_json.to_string())
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;
    use crate::{
        common::test_utils::mock_profile,
        utils::{constants::*, devsetup::*},
    };

    #[tokio::test]
    async fn test_build_cred_defs_json_verifier_with_multiple_credentials() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfoVerifier {
            schema_id: "schema_key1".to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let cred2 = CredInfoVerifier {
            schema_id: "schema_key2".to_string(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let credentials = vec![cred1, cred2];
        let credential_json = build_cred_defs_json_verifier(&mock_profile(), &credentials)
            .await
            .unwrap();

        let json: Value = serde_json::from_str(CRED_DEF_JSON).unwrap();
        let expected = json!({ CRED_DEF_ID: json }).to_string();
        assert_eq!(credential_json, expected);
    }

    #[tokio::test]
    async fn test_build_schemas_json_verifier_with_multiple_schemas() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfoVerifier {
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: "cred_def_key1".to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let cred2 = CredInfoVerifier {
            schema_id: SCHEMA_ID.to_string(),
            cred_def_id: "cred_def_key2".to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let credentials = vec![cred1, cred2];
        let schema_json = build_schemas_json_verifier(&mock_profile(), &credentials)
            .await
            .unwrap();

        let json: Value = serde_json::from_str(SCHEMA_JSON).unwrap();
        let expected = json!({ SCHEMA_ID: json }).to_string();
        assert_eq!(schema_json, expected);
    }

    #[tokio::test]
    async fn test_build_rev_reg_defs_json() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfoVerifier {
            schema_id: "schema_key1".to_string(),
            cred_def_id: "cred_def_key1".to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            timestamp: None,
        };
        let cred2 = CredInfoVerifier {
            schema_id: "schema_key2".to_string(),
            cred_def_id: "cred_def_key2".to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            timestamp: None,
        };
        let credentials = vec![cred1, cred2];
        let rev_reg_defs_json = build_rev_reg_defs_json(&mock_profile(), &credentials).await.unwrap();

        let json: Value = serde_json::from_str(&rev_def_json()).unwrap();
        let expected = json!({ REV_REG_ID: json }).to_string();
        assert_eq!(rev_reg_defs_json, expected);
    }

    #[tokio::test]
    async fn test_build_rev_reg_json() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfoVerifier {
            schema_id: "schema_key1".to_string(),
            cred_def_id: "cred_def_key1".to_string(),
            rev_reg_id: Some("id1".to_string()),
            timestamp: Some(1),
        };
        let cred2 = CredInfoVerifier {
            schema_id: "schema_key2".to_string(),
            cred_def_id: "cred_def_key2".to_string(),
            rev_reg_id: Some("id2".to_string()),
            timestamp: Some(2),
        };
        let credentials = vec![cred1, cred2];
        let rev_reg_json = build_rev_reg_json(&mock_profile(), &credentials).await.unwrap();

        let json: Value = serde_json::from_str(REV_REG_JSON).unwrap();
        let expected = json!({REV_REG_ID:{"1":json}}).to_string();
        assert_eq!(rev_reg_json, expected);
    }
}
