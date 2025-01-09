use anoncreds_types::data_types::identifiers::schema_id::SchemaId;
use aries_vcx_ledger::ledger::base_ledger::AnoncredsLedgerRead;
use serde_json::{self, Value};

use crate::{errors::error::prelude::*, utils::openssl::encode};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CredInfoVerifier {
    pub schema_id: SchemaId,
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
            if let (Some(schema_id), Some(cred_def_id)) = (
                identifier["schema_id"].as_str(),
                identifier["cred_def_id"].as_str(),
            ) {
                let rev_reg_id = identifier["rev_reg_id"].as_str().map(|x| x.to_string());

                let timestamp = identifier["timestamp"].as_u64();
                rtn.push(CredInfoVerifier {
                    schema_id: SchemaId::new(schema_id)?,
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
            format!(
                "Cannot get encoded value for \"{}\" attribute",
                attr1_referent
            ),
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
    ledger: &impl AnoncredsLedgerRead,
    credential_data: &[CredInfoVerifier],
) -> VcxResult<String> {
    trace!("build_cred_defs_json_verifier >>");
    let mut credential_json = json!({});

    for cred_info in credential_data.iter() {
        if credential_json.get(&cred_info.cred_def_id).is_none() {
            let cred_def_id = &cred_info.cred_def_id;
            let credential_def = ledger
                .get_cred_def(&cred_def_id.to_string().try_into()?, None)
                .await?;

            let credential_def = serde_json::to_value(&credential_def).map_err(|err| {
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
    ledger: &impl AnoncredsLedgerRead,
    credential_data: &[CredInfoVerifier],
) -> VcxResult<String> {
    trace!("build_schemas_json_verifier >>");

    let mut schemas_json = json!({});

    for cred_info in credential_data.iter() {
        if schemas_json.get(cred_info.schema_id.to_string()).is_none() {
            let schema_id = &cred_info.schema_id;
            let schema_json = ledger.get_schema(schema_id, None).await.map_err(|_err| {
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidSchema, "Cannot get schema")
            })?;
            let schema_val = serde_json::to_value(&schema_json).map_err(|err| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidSchema,
                    format!("Cannot deserialize schema: {}", err),
                )
            })?;
            schemas_json[schema_id.to_string()] = schema_val;
        }
    }

    Ok(schemas_json.to_string())
}

pub async fn build_rev_reg_defs_json(
    ledger: &impl AnoncredsLedgerRead,
    credential_data: &[CredInfoVerifier],
) -> VcxResult<String> {
    trace!("build_rev_reg_defs_json >>");

    let mut rev_reg_defs_json = json!({});

    for cred_info in credential_data.iter().filter(|r| r.rev_reg_id.is_some()) {
        let rev_reg_id = cred_info
            .rev_reg_id
            .as_ref()
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidRevocationDetails,
                format!(
                    "build_rev_reg_defs_json >> Missing rev_reg_id in the record {:?}",
                    cred_info
                ),
            ))?;

        if rev_reg_defs_json.get(rev_reg_id).is_none() {
            let (json, _meta) = ledger
                .get_rev_reg_def_json(&rev_reg_id.to_string().try_into()?)
                .await?;
            let rev_reg_def_json = serde_json::to_value(&json).or(Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Failed to deserialize as json rev_reg_def: {:?}", json),
            )))?;
            rev_reg_defs_json[rev_reg_id] = rev_reg_def_json;
        }
    }

    Ok(rev_reg_defs_json.to_string())
}

pub async fn build_rev_reg_json(
    ledger: &impl AnoncredsLedgerRead,
    credential_data: &[CredInfoVerifier],
) -> VcxResult<String> {
    trace!("build_rev_reg_json >>");

    let mut rev_regs_json = json!({});

    for cred_info in credential_data.iter().filter(|r| r.rev_reg_id.is_some()) {
        let rev_reg_id = cred_info
            .rev_reg_id
            .as_ref()
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidRevocationDetails,
                format!(
                    "build_rev_reg_json >> missing rev_reg_id in the record {:?}",
                    cred_info
                ),
            ))?;

        let timestamp = cred_info.timestamp.as_ref().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidRevocationTimestamp,
            format!("Revocation timestamp is missing on record {:?}", cred_info),
        ))?;

        if rev_regs_json.get(rev_reg_id).is_none() {
            let (rev_reg_json, timestamp) = ledger
                .get_rev_reg(&rev_reg_id.to_owned().try_into()?, timestamp.to_owned())
                .await?;
            let rev_reg_json =
                serde_json::to_value(rev_reg_json.clone()).or(Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidJson,
                    format!("Failed to deserialize as json: {:?}", rev_reg_json),
                )))?;
            let rev_reg_json = json!({ timestamp.to_string(): rev_reg_json });
            rev_regs_json[rev_reg_id] = rev_reg_json;
        }
    }

    Ok(rev_regs_json.to_string())
}

#[cfg(test)]
pub mod unit_tests {
    use anoncreds_types::data_types::ledger::cred_def::CredentialDefinition;
    use test_utils::{constants::*, devsetup::*, mockdata::mock_ledger::MockLedger};

    use super::*;

    #[tokio::test]
    async fn test_build_cred_defs_json_verifier_with_multiple_credentials() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfoVerifier {
            schema_id: schema_id(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let cred2 = CredInfoVerifier {
            schema_id: schema_id(),
            cred_def_id: CRED_DEF_ID.to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let credentials = vec![cred1, cred2];
        let ledger_read = MockLedger;
        let credential_json = build_cred_defs_json_verifier(&ledger_read, &credentials)
            .await
            .unwrap();

        let json: CredentialDefinition = serde_json::from_str(CRED_DEF_JSON).unwrap();
        let expected = json!({ CRED_DEF_ID: json }).to_string();
        assert_eq!(credential_json, expected);
    }

    #[tokio::test]
    async fn test_build_schemas_json_verifier_with_multiple_schemas() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfoVerifier {
            schema_id: schema_id(),
            cred_def_id: "cred_def_key1".to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let cred2 = CredInfoVerifier {
            schema_id: schema_id(),
            cred_def_id: "cred_def_key2".to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let ledger_read = MockLedger;
        let credentials = vec![cred1, cred2];
        let schema_json = build_schemas_json_verifier(&ledger_read, &credentials)
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
            schema_id: schema_id(),
            cred_def_id: "cred_def_key1".to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            timestamp: None,
        };
        let cred2 = CredInfoVerifier {
            schema_id: schema_id(),
            cred_def_id: "cred_def_key2".to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            timestamp: None,
        };
        let ledger_read = MockLedger;
        let credentials = vec![cred1, cred2];
        let rev_reg_defs_json = build_rev_reg_defs_json(&ledger_read, &credentials)
            .await
            .unwrap();

        let json: Value = serde_json::to_value(rev_def_json()).unwrap();
        let expected = json!({ REV_REG_ID: json }).to_string();
        assert_eq!(rev_reg_defs_json, expected);
    }

    #[tokio::test]
    async fn test_build_rev_reg_json() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfoVerifier {
            schema_id: schema_id(),
            cred_def_id: "cred_def_key1".to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            timestamp: Some(1),
        };
        let cred2 = CredInfoVerifier {
            schema_id: schema_id(),
            cred_def_id: "cred_def_key2".to_string(),
            rev_reg_id: Some(REV_REG_ID.to_string()),
            timestamp: Some(2),
        };
        let ledger_read = MockLedger;
        let credentials = vec![cred1, cred2];
        let rev_reg_json = build_rev_reg_json(&ledger_read, &credentials)
            .await
            .unwrap();

        let json: Value = serde_json::from_str(REV_REG_JSON).unwrap();
        let expected = json!({REV_REG_ID:{"1":json}}).to_string();
        assert_eq!(rev_reg_json, expected);
    }
}
