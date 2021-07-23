use crate::error::prelude::*;
use crate::libindy::proofs::verifier::verifier_internal::{build_cred_defs_json_verifier, build_rev_reg_defs_json, build_rev_reg_json, build_schemas_json_verifier, get_credential_info, validate_proof_revealed_attributes};
use crate::libindy::utils::anoncreds;
use crate::utils::mockdata::mock_settings::get_mock_result_for_validate_indy_proof;

pub fn validate_indy_proof(proof_json: &str, proof_req_json: &str) -> VcxResult<bool> {
    if let Some(mock_result) = get_mock_result_for_validate_indy_proof() {
        return mock_result;
    }

    validate_proof_revealed_attributes(&proof_json)?;

    let credential_data = get_credential_info(&proof_json)?;

    let credential_defs_json = build_cred_defs_json_verifier(&credential_data)
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
    use crate::{libindy, utils, settings};
    use crate::libindy::proofs::proof_request::ProofRequestData;
    use crate::utils::devsetup::SetupLibraryWalletPoolZeroFees;

    use super::*;

    #[test]
    #[cfg(feature = "pool_tests")]
    fn test_proof_self_attested_proof_validation() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

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

        let proof_req_json = ProofRequestData::create()
            .set_name(name)
            .set_requested_attributes(requested_attrs).unwrap()
            .set_requested_predicates(requested_predicates).unwrap()
            .set_not_revoked_interval(revocation_details).unwrap()
            .set_nonce().unwrap();

        let proof_req_json = serde_json::to_string(&proof_req_json).unwrap();

        let prover_proof_json = libindy::utils::anoncreds::libindy_prover_create_proof(
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

        let proof_req_json = ProofRequestData::create()
            .set_name(name)
            .set_requested_attributes(requested_attrs).unwrap()
            .set_requested_predicates(requested_predicates).unwrap()
            .set_not_revoked_interval(revocation_details).unwrap()
            .set_nonce().unwrap();

        let proof_req_json = serde_json::to_string(&proof_req_json).unwrap();

        let (schema_id, schema_json, cred_def_id, cred_def_json, _offer, _req, _req_meta, cred_id, _, _)
            = libindy::utils::anoncreds::tests::create_and_store_credential(utils::constants::DEFAULT_SCHEMA_ATTRS, false);
        let cred_def_json: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let schema_json: serde_json::Value = serde_json::from_str(&schema_json).unwrap();

        let prover_proof_json = libindy::utils::anoncreds::libindy_prover_create_proof(
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

        let proof_req_json = ProofRequestData::create()
            .set_name(name)
            .set_requested_attributes(requested_attrs).unwrap()
            .set_requested_predicates(requested_predicates).unwrap()
            .set_not_revoked_interval(revocation_details).unwrap()
            .set_nonce().unwrap();

        let proof_req_json = serde_json::to_string(&proof_req_json).unwrap();

        let (schema_id, schema_json, cred_def_id, cred_def_json, _offer, _req, _req_meta, cred_id, _, _)
            = libindy::utils::anoncreds::tests::create_and_store_credential(utils::constants::DEFAULT_SCHEMA_ATTRS, false);
        let cred_def_json: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let schema_json: serde_json::Value = serde_json::from_str(&schema_json).unwrap();

        let prover_proof_json = libindy::utils::anoncreds::libindy_prover_create_proof(
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

