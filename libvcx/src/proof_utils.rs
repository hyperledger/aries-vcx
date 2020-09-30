use serde_json;
use serde_json::Value;

use error::prelude::*;
use messages::proofs::proof_message::{CredInfoVerifier,
    CredInfoProver,
    get_credential_info
};
use object_cache::ObjectCache;
use settings;
use settings::get_config_value;
use utils::error;
use utils::libindy::anoncreds;
use utils::openssl::encode;
use v3::handlers::proof_presentation::verifier::verifier::Verifier;
use utils::mockdata::mock_settings::get_mock_result_for_validate_indy_proof;

fn validate_proof_revealed_attributes(proof_json: &str) -> VcxResult<()> {
    if settings::indy_mocks_enabled() { return Ok(()); }

    let proof: Value = serde_json::from_str(proof_json)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize libndy proof: {}", err)))?;

    let revealed_attrs = match proof["requested_proof"]["revealed_attrs"].as_object() {
        Some(revealed_attrs) => revealed_attrs,
        None => return Ok(())
    };

    for (attr1_referent, info) in revealed_attrs.iter() {
        let raw = info["raw"].as_str().ok_or(VcxError::from_msg(VcxErrorKind::InvalidProof, format!("Cannot get raw value for \"{}\" attribute", attr1_referent)))?;
        let encoded_ = info["encoded"].as_str().ok_or(VcxError::from_msg(VcxErrorKind::InvalidProof, format!("Cannot get encoded value for \"{}\" attribute", attr1_referent)))?;

        let expected_encoded = encode(&raw)?;

        if expected_encoded != encoded_.to_string() {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidProof, format!("Encoded values are different. Expected: {}. From Proof: {}", expected_encoded, encoded_)));
        }
    }

    Ok(())
}

fn build_credential_defs_json(credential_data: &Vec<CredInfoVerifier>) -> VcxResult<String> {
    debug!("building credential_def_json for proof validation");
    let mut credential_json = json!({});

    for ref cred_info in credential_data.iter() {
        if credential_json.get(&cred_info.cred_def_id).is_none() {
            let (id, credential_def) = anoncreds::get_cred_def_json(&cred_info.cred_def_id)?;

            let credential_def = serde_json::from_str(&credential_def)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidProofCredentialData, format!("Cannot deserialize credential definition: {}", err)))?;

            credential_json[id] = credential_def;
        }
    }

    Ok(credential_json.to_string())
}

fn build_schemas_json_verifier(credential_data: &Vec<CredInfoVerifier>) -> VcxResult<String> {
    debug!("building schemas json for proof validation");

    let mut schemas_json = json!({});

    for ref cred_info in credential_data.iter() {
        if schemas_json.get(&cred_info.schema_id).is_none() {
            let (id, schema_json) = anoncreds::get_schema_json(&cred_info.schema_id)
                .map_err(|err| err.map(VcxErrorKind::InvalidSchema, "Cannot get schema"))?;

            let schema_val = serde_json::from_str(&schema_json)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidSchema, format!("Cannot deserialize schema: {}", err)))?;

            schemas_json[id] = schema_val;
        }
    }

    Ok(schemas_json.to_string())
}

pub fn build_schemas_json_prover(credentials_identifiers: &Vec<CredInfoProver>) -> VcxResult<String> {
    let mut rtn: Value = json!({});

    for ref cred_info in credentials_identifiers {
        if rtn.get(&cred_info.schema_id).is_none() {
            let (_, schema_json) = anoncreds::get_schema_json(&cred_info.schema_id)
                .map_err(|err| err.map(VcxErrorKind::InvalidSchema, "Cannot get schema"))?;

            let schema_json = serde_json::from_str(&schema_json)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidSchema, format!("Cannot deserialize schema: {}", err)))?;

            rtn[cred_info.schema_id.to_owned()] = schema_json;
        }
    }
    Ok(rtn.to_string())
}


fn build_rev_reg_defs_json(credential_data: &Vec<CredInfoVerifier>) -> VcxResult<String> {
    debug!("building rev_reg_def_json for proof validation");

    let mut rev_reg_defs_json = json!({});

    for ref cred_info in credential_data.iter() {
        let rev_reg_id = cred_info
            .rev_reg_id
            .as_ref()
            .ok_or(VcxError::from(VcxErrorKind::InvalidRevocationDetails))?;

        if rev_reg_defs_json.get(rev_reg_id).is_none() {
            let (id, json) = anoncreds::get_rev_reg_def_json(rev_reg_id)
                .or(Err(VcxError::from(VcxErrorKind::InvalidRevocationDetails)))?;

            let rev_reg_def_json = serde_json::from_str(&json)
                .or(Err(VcxError::from(VcxErrorKind::InvalidSchema)))?;

            rev_reg_defs_json[id] = rev_reg_def_json;
        }
    }

    Ok(rev_reg_defs_json.to_string())
}

fn build_rev_reg_json(credential_data: &Vec<CredInfoVerifier>) -> VcxResult<String> {
    debug!("building rev_reg_json for proof validation");

    let mut rev_regs_json = json!({});

    for ref cred_info in credential_data.iter() {
        let rev_reg_id = cred_info
            .rev_reg_id
            .as_ref()
            .ok_or(VcxError::from(VcxErrorKind::InvalidRevocationDetails))?;

        let timestamp = cred_info
            .timestamp
            .as_ref()
            .ok_or(VcxError::from(VcxErrorKind::InvalidRevocationTimestamp))?;

        if rev_regs_json.get(rev_reg_id).is_none() {
            let (id, json, timestamp) = anoncreds::get_rev_reg(rev_reg_id, timestamp.to_owned())
                .or(Err(VcxError::from(VcxErrorKind::InvalidRevocationDetails)))?;

            let rev_reg_json: Value = serde_json::from_str(&json)
                .or(Err(VcxError::from(VcxErrorKind::InvalidJson)))?;

            let rev_reg_json = json!({timestamp.to_string(): rev_reg_json});
            rev_regs_json[id] = rev_reg_json;
        }
    }

    Ok(rev_regs_json.to_string())
}

pub fn validate_indy_proof(proof_json: &str, proof_req_json: &str) -> VcxResult<bool> {
    if let Some(mock_result) = get_mock_result_for_validate_indy_proof() {
        return mock_result
    }

    validate_proof_revealed_attributes(&proof_json)?;

    let credential_data = get_credential_info(&proof_json)?;

    let credential_defs_json = build_credential_defs_json(&credential_data)
        .unwrap_or(json!({}).to_string());
    let schemas_json = build_schemas_json_verifier(&credential_data)
        .unwrap_or(json!({}).to_string());
    let rev_reg_defs_json = build_rev_reg_defs_json(&credential_data)
        .unwrap_or(json!({}).to_string());
    let rev_regs_json = build_rev_reg_json(&credential_data)
        .unwrap_or(json!({}).to_string());

    debug!("*******\n{}\n********", credential_defs_json);
    debug!("*******\n{}\n********", schemas_json);
    debug!("*******\n{}\n********", proof_json);
    debug!("*******\n{}\n********", proof_req_json);
    debug!("*******\n{}\n********", rev_reg_defs_json);
    debug!("*******\n{}\n********", rev_regs_json);
    anoncreds::libindy_verifier_verify_proof(proof_req_json,
                                             proof_json,
                                             &schemas_json,
                                             &credential_defs_json,
                                             &rev_reg_defs_json,
                                             &rev_regs_json)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use api::VcxStateType;
    use connection::tests::build_test_connection_inviter_requested;

    use utils::devsetup::*;
    use utils::constants::*;
    use utils::httpclient::HttpClientMockResponse;
    use utils::mockdata::mockdata_proof;
    use v3::handlers::proof_presentation::verifier::verifier::Verifier;
    use v3::messages::proof_presentation::presentation_request::PresentationRequestData;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_build_credential_defs_json_with_multiple_credentials() {
        let _setup = SetupStrictAriesMocks::init();

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
        let credential_json = build_credential_defs_json(&credentials).unwrap();

        let json: Value = serde_json::from_str(CRED_DEF_JSON).unwrap();
        let expected = json!({CRED_DEF_ID:json}).to_string();
        assert_eq!(credential_json, expected);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_build_schemas_json_verifier_with_multiple_schemas() {
        let _setup = SetupStrictAriesMocks::init();

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
        let schema_json = build_schemas_json_verifier(&credentials).unwrap();

        let json: Value = serde_json::from_str(SCHEMA_JSON).unwrap();
        let expected = json!({SCHEMA_ID:json}).to_string();
        assert_eq!(schema_json, expected);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_build_rev_reg_defs_json() {
        let _setup = SetupStrictAriesMocks::init();

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
        let rev_reg_defs_json = build_rev_reg_defs_json(&credentials).unwrap();

        let json: Value = serde_json::from_str(&rev_def_json()).unwrap();
        let expected = json!({REV_REG_ID:json}).to_string();
        assert_eq!(rev_reg_defs_json, expected);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_build_rev_reg_json() {
        let _setup = SetupStrictAriesMocks::init();

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
        let rev_reg_json = build_rev_reg_json(&credentials).unwrap();

        let json: Value = serde_json::from_str(REV_REG_JSON).unwrap();
        let expected = json!({REV_REG_ID:{"1":json}}).to_string();
        assert_eq!(rev_reg_json, expected);
    }

    #[test]
    #[cfg(feature = "pool_tests")]
    fn test_proof_self_attested_proof_validation() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();
        settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, "4.0");

        let requested_attrs = json!([
                                            json!({
                                                "name":"address1",
                                                "self_attest_allowed": true,
                                            }),
                                            json!({
                                                "name":"zip",
                                                "self_attest_allowed": true,
                                            }),
                                         ]).to_string();
        let requested_predicates = json!([]).to_string();
        let revocation_details = r#"{"support_revocation":false}"#.to_string();
        let name = "Optional".to_owned();

        let proof_req_json = PresentationRequestData::create()
            .set_name(name)
            .set_requested_attributes(requested_attrs).unwrap()
            .set_requested_predicates(requested_predicates).unwrap()
            .set_not_revoked_interval(revocation_details).unwrap()
            .set_nonce().unwrap();

        let proof_req_json = serde_json::to_string(&proof_req_json).unwrap();

        let prover_proof_json = ::utils::libindy::anoncreds::libindy_prover_create_proof(
            &proof_req_json,
            &json!({
              "self_attested_attributes":{
                 "attribute_0": "my_self_attested_address",
                 "attribute_1": "my_self_attested_zip"
              },
              "requested_attributes":{},
              "requested_predicates":{}
            }).to_string(),
            "main",
            &json!({}).to_string(),
            &json!({}).to_string(),
            None).unwrap();

        assert_eq!(validate_indy_proof(&prover_proof_json, &proof_req_json.to_string()).unwrap(), true);
    }

    #[test]
    #[cfg(feature = "pool_tests")]
    fn test_proof_restrictions() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();
        ::settings::set_config_value(::settings::CONFIG_PROTOCOL_TYPE, "4.0");

        let requested_attrs = json!([
                                            json!({
                                                "name":"address1",
                                                "restrictions": [{ "issuer_did": "Not Here" }],
                                            }),
                                            json!({
                                                "name":"zip",
                                            }),
                                            json!({
                                                "name":"self_attest",
                                                "self_attest_allowed": true,
                                            }),
                                         ]).to_string();
        let requested_predicates = json!([]).to_string();
        let revocation_details = r#"{"support_revocation":true}"#.to_string();
        let name = "Optional".to_owned();

        let proof_req_json = PresentationRequestData::create()
            .set_name(name)
            .set_requested_attributes(requested_attrs).unwrap()
            .set_requested_predicates(requested_predicates).unwrap()
            .set_not_revoked_interval(revocation_details).unwrap()
            .set_nonce().unwrap();

        let proof_req_json = serde_json::to_string(&proof_req_json).unwrap();

        let (schema_id, schema_json, cred_def_id, cred_def_json, _offer, _req, _req_meta, cred_id, _, _)
            = ::utils::libindy::anoncreds::tests::create_and_store_credential(::utils::constants::DEFAULT_SCHEMA_ATTRS, false);
        let cred_def_json: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let schema_json: serde_json::Value = serde_json::from_str(&schema_json).unwrap();

        let prover_proof_json = ::utils::libindy::anoncreds::libindy_prover_create_proof(
            &proof_req_json,
            &json!({
                "self_attested_attributes":{
                   "attribute_2": "my_self_attested_val"
                },
                "requested_attributes":{
                   "attribute_0": {"cred_id": cred_id, "revealed": true},
                   "attribute_1": {"cred_id": cred_id, "revealed": true}
                },
                "requested_predicates":{}
            }).to_string(),
            "main",
            &json!({schema_id: schema_json}).to_string(),
            &json!({cred_def_id: cred_def_json}).to_string(),
            None).unwrap();
        assert_eq!(validate_indy_proof(&prover_proof_json, &proof_req_json).unwrap_err().kind(), VcxErrorKind::LibndyError(405)); // AnoncredsProofRejected

        let mut proof_req_json: serde_json::Value = serde_json::from_str(&proof_req_json).unwrap();
        proof_req_json["requested_attributes"]["attribute_0"]["restrictions"] = json!({});
        assert_eq!(validate_indy_proof(&prover_proof_json, &proof_req_json.to_string()).unwrap(), true);
    }

    #[test]
    #[cfg(feature = "pool_tests")]
    fn test_proof_validate_attribute() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();
        ::settings::set_config_value(::settings::CONFIG_PROTOCOL_TYPE, "4.0");

        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let requested_attrs = json!([
                                            json!({
                                                "name":"address1",
                                                "restrictions": [json!({ "issuer_did": did })]
                                            }),
                                            json!({
                                                "name":"zip",
                                                "restrictions": [json!({ "issuer_did": did })]
                                            }),
                                            json!({
                                                "name":"self_attest",
                                                "self_attest_allowed": true,
                                            }),
                                         ]).to_string();
        let requested_predicates = json!([]).to_string();
        let revocation_details = r#"{"support_revocation":true}"#.to_string();
        let name = "Optional".to_owned();

        let proof_req_json = PresentationRequestData::create()
            .set_name(name)
            .set_requested_attributes(requested_attrs).unwrap()
            .set_requested_predicates(requested_predicates).unwrap()
            .set_not_revoked_interval(revocation_details).unwrap()
            .set_nonce().unwrap();

        let proof_req_json = serde_json::to_string(&proof_req_json).unwrap();

        let (schema_id, schema_json, cred_def_id, cred_def_json, _offer, _req, _req_meta, cred_id, _, _)
            = ::utils::libindy::anoncreds::tests::create_and_store_credential(::utils::constants::DEFAULT_SCHEMA_ATTRS, false);
        let cred_def_json: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let schema_json: serde_json::Value = serde_json::from_str(&schema_json).unwrap();

        let prover_proof_json = ::utils::libindy::anoncreds::libindy_prover_create_proof(
            &proof_req_json,
            &json!({
                "self_attested_attributes":{
                   "attribute_2": "my_self_attested_val"
                },
                "requested_attributes":{
                   "attribute_0": {"cred_id": cred_id, "revealed": true},
                   "attribute_1": {"cred_id": cred_id, "revealed": true}
                },
                "requested_predicates":{}
            }).to_string(),
            "main",
            &json!({schema_id: schema_json}).to_string(),
            &json!({cred_def_id: cred_def_json}).to_string(),
            None).unwrap();
        assert_eq!(validate_indy_proof(&prover_proof_json, &proof_req_json).unwrap(), true);

        let mut proof_obj: serde_json::Value = serde_json::from_str(&prover_proof_json).unwrap();
        {
            proof_obj["requested_proof"]["revealed_attrs"]["address1_1"]["raw"] = json!("Other Value");
            let prover_proof_json = serde_json::to_string(&proof_obj).unwrap();

            assert_eq!(validate_indy_proof(&prover_proof_json, &proof_req_json).unwrap_err().kind(), VcxErrorKind::InvalidProof);
        }
        {
            proof_obj["requested_proof"]["revealed_attrs"]["address1_1"]["encoded"] = json!("1111111111111111111111111111111111111111111111111111111111");
            let prover_proof_json = serde_json::to_string(&proof_obj).unwrap();

            assert_eq!(validate_indy_proof(&prover_proof_json, &proof_req_json).unwrap_err().kind(), VcxErrorKind::InvalidProof);
        }
    }
}
