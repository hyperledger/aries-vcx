use indy_sys::{WalletHandle, PoolHandle};
use serde_json;
use serde_json::Value;

use crate::error::prelude::*;
use crate::global::settings;
use crate::libindy::utils::anoncreds;
use crate::utils::openssl::encode;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct CredInfoVerifier {
    pub schema_id: String,
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub timestamp: Option<u64>,
}

pub fn get_credential_info(proof: &str) -> VcxResult<Vec<CredInfoVerifier>> {
    let mut rtn = Vec::new();

    let credentials: Value = serde_json::from_str(proof).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::InvalidJson,
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
                return Err(VcxError::from_msg(
                    VcxErrorKind::InvalidProofCredentialData,
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
        VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("Cannot deserialize libndy proof: {}", err),
        )
    })?;

    let revealed_attrs = match proof["requested_proof"]["revealed_attrs"].as_object() {
        Some(revealed_attrs) => revealed_attrs,
        None => return Ok(()),
    };

    for (attr1_referent, info) in revealed_attrs.iter() {
        let raw = info["raw"].as_str().ok_or(VcxError::from_msg(
            VcxErrorKind::InvalidProof,
            format!("Cannot get raw value for \"{}\" attribute", attr1_referent),
        ))?;
        let encoded_ = info["encoded"].as_str().ok_or(VcxError::from_msg(
            VcxErrorKind::InvalidProof,
            format!("Cannot get encoded value for \"{}\" attribute", attr1_referent),
        ))?;

        let expected_encoded = encode(raw)?;

        if expected_encoded != *encoded_ {
            return Err(VcxError::from_msg(
                VcxErrorKind::InvalidProof,
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
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    credential_data: &Vec<CredInfoVerifier>,
) -> VcxResult<String> {
    debug!("building credential_def_json for proof validation");
    let mut credential_json = json!({});

    for cred_info in credential_data.iter() {
        if credential_json.get(&cred_info.cred_def_id).is_none() {
            let (id, credential_def) = anoncreds::get_cred_def_json(wallet_handle, pool_handle, &cred_info.cred_def_id).await?;

            let credential_def = serde_json::from_str(&credential_def).map_err(|err| {
                VcxError::from_msg(
                    VcxErrorKind::InvalidProofCredentialData,
                    format!("Cannot deserialize credential definition: {}", err),
                )
            })?;

            credential_json[id] = credential_def;
        }
    }

    Ok(credential_json.to_string())
}

pub async fn build_schemas_json_verifier(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    credential_data: &Vec<CredInfoVerifier>,
) -> VcxResult<String> {
    debug!("building schemas json for proof validation");

    let mut schemas_json = json!({});

    for cred_info in credential_data.iter() {
        if schemas_json.get(&cred_info.schema_id).is_none() {
            let (id, schema_json) = anoncreds::get_schema_json(wallet_handle, pool_handle, &cred_info.schema_id)
                .await
                .map_err(|err| err.map(VcxErrorKind::InvalidSchema, "Cannot get schema"))?;

            let schema_val = serde_json::from_str(&schema_json).map_err(|err| {
                VcxError::from_msg(
                    VcxErrorKind::InvalidSchema,
                    format!("Cannot deserialize schema: {}", err),
                )
            })?;

            schemas_json[id] = schema_val;
        }
    }

    Ok(schemas_json.to_string())
}

pub async fn build_rev_reg_defs_json(credential_data: &Vec<CredInfoVerifier>) -> VcxResult<String> {
    debug!("building rev_reg_def_json for proof validation");

    let mut rev_reg_defs_json = json!({});

    for cred_info in credential_data.iter() {
        let rev_reg_id = cred_info
            .rev_reg_id
            .as_ref()
            .ok_or(VcxError::from(VcxErrorKind::InvalidRevocationDetails))?;

        let pool_handle = crate::global::pool::get_main_pool_handle()?;
        if rev_reg_defs_json.get(rev_reg_id).is_none() {
            let (id, json) = anoncreds::get_rev_reg_def_json(pool_handle, rev_reg_id)
                .await
                .or(Err(VcxError::from(VcxErrorKind::InvalidRevocationDetails)))?;

            let rev_reg_def_json = serde_json::from_str(&json).or(Err(VcxError::from(VcxErrorKind::InvalidSchema)))?;

            rev_reg_defs_json[id] = rev_reg_def_json;
        }
    }

    Ok(rev_reg_defs_json.to_string())
}

pub async fn build_rev_reg_json(credential_data: &Vec<CredInfoVerifier>) -> VcxResult<String> {
    debug!("building rev_reg_json for proof validation");

    let mut rev_regs_json = json!({});

    for cred_info in credential_data.iter() {
        let rev_reg_id = cred_info
            .rev_reg_id
            .as_ref()
            .ok_or(VcxError::from(VcxErrorKind::InvalidRevocationDetails))?;

        let timestamp = cred_info
            .timestamp
            .as_ref()
            .ok_or(VcxError::from(VcxErrorKind::InvalidRevocationTimestamp))?;

        let pool_handle = crate::global::pool::get_main_pool_handle()?;
        if rev_regs_json.get(rev_reg_id).is_none() {
            let (id, json, timestamp) = anoncreds::get_rev_reg(pool_handle, rev_reg_id, timestamp.to_owned())
                .await
                .or(Err(VcxError::from(VcxErrorKind::InvalidRevocationDetails)))?;

            let rev_reg_json: Value = serde_json::from_str(&json).or(Err(VcxError::from(VcxErrorKind::InvalidJson)))?;

            let rev_reg_json = json!({ timestamp.to_string(): rev_reg_json });
            rev_regs_json[id] = rev_reg_json;
        }
    }

    Ok(rev_regs_json.to_string())
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use crate::utils::constants::*;
    use crate::utils::devsetup::*;

    use super::*;

    #[tokio::test]
    async fn test_build_cred_defs_json_verifier_with_multiple_credentials() {
        let _setup = SetupMocks::init();

        let cred1 = CredInfoVerifier {
            schema_id: "schema_key1".to_string(),
            cred_def_id: "cred_def_key1".to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let cred2 = CredInfoVerifier {
            schema_id: "schema_key2".to_string(),
            cred_def_id: "cred_def_key2".to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let credentials = vec![cred1, cred2];
        let credential_json = build_cred_defs_json_verifier(WalletHandle(0), 0, &credentials)
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
            schema_id: "schema_key1".to_string(),
            cred_def_id: "cred_def_key1".to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let cred2 = CredInfoVerifier {
            schema_id: "schema_key2".to_string(),
            cred_def_id: "cred_def_key2".to_string(),
            rev_reg_id: None,
            timestamp: None,
        };
        let credentials = vec![cred1, cred2];
        let schema_json = build_schemas_json_verifier(WalletHandle(0), 0, &credentials)
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
            rev_reg_id: Some("id1".to_string()),
            timestamp: None,
        };
        let cred2 = CredInfoVerifier {
            schema_id: "schema_key2".to_string(),
            cred_def_id: "cred_def_key2".to_string(),
            rev_reg_id: Some("id2".to_string()),
            timestamp: None,
        };
        let credentials = vec![cred1, cred2];
        let rev_reg_defs_json = build_rev_reg_defs_json(&credentials).await.unwrap();

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
        let rev_reg_json = build_rev_reg_json(&credentials).await.unwrap();

        let json: Value = serde_json::from_str(REV_REG_JSON).unwrap();
        let expected = json!({REV_REG_ID:{"1":json}}).to_string();
        assert_eq!(rev_reg_json, expected);
    }
}
