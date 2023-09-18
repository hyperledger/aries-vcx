use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use std::sync::Arc;

use crate::common::proofs::verifier::verifier_internal::{
    build_cred_defs_json_verifier, build_rev_reg_defs_json, build_rev_reg_json, build_schemas_json_verifier,
    get_credential_info, validate_proof_revealed_attributes,
};
use crate::errors::error::prelude::*;
use crate::utils::mockdata::mock_settings::get_mock_result_for_validate_indy_proof;

pub async fn validate_indy_proof(
    ledger: &Arc<dyn AnoncredsLedgerRead>,
    anoncreds: &Arc<dyn BaseAnonCreds>,
    proof_json: &str,
    proof_req_json: &str,
) -> VcxResult<bool> {
    if let Some(mock_result) = get_mock_result_for_validate_indy_proof() {
        return mock_result;
    }
    validate_proof_revealed_attributes(proof_json)?;

    let credential_data = get_credential_info(proof_json)?;
    debug!("validate_indy_proof >> credential_data: {credential_data:?}");
    let credential_defs_json = build_cred_defs_json_verifier(ledger, &credential_data).await?;
    let schemas_json = build_schemas_json_verifier(ledger, &credential_data).await?;
    let rev_reg_defs_json = build_rev_reg_defs_json(ledger, &credential_data)
        .await
        .unwrap_or(json!({}).to_string());
    let rev_regs_json = build_rev_reg_json(ledger, &credential_data)
        .await
        .unwrap_or(json!({}).to_string());

    debug!("validate_indy_proof >> credential_defs_json: {credential_defs_json}");
    debug!("validate_indy_proof >> schemas_json: {schemas_json}");
    trace!("validate_indy_proof >> proof_json: {proof_json}");
    debug!("validate_indy_proof >> proof_req_json: {proof_req_json}");
    debug!("validate_indy_proof >> rev_reg_defs_json: {rev_reg_defs_json}");
    debug!("validate_indy_proof >> rev_regs_json: {rev_regs_json}");
    anoncreds
        .verifier_verify_proof(
            proof_req_json,
            proof_json,
            &schemas_json,
            &credential_defs_json,
            &rev_reg_defs_json,
            &rev_regs_json,
        )
        .await
        .map_err(|err| err.into())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod integration_tests {
    use super::*;

    use std::{sync::Arc, time::Duration};

    use aries_vcx_core::{
        anoncreds::base_anoncreds::BaseAnonCreds,
        ledger::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite},
    };

    use crate::{
        common::{
            proofs::proof_request::ProofRequestData,
            test_utils::{create_and_write_credential, create_and_write_test_cred_def, create_and_write_test_schema},
        },
        errors::error::AriesVcxErrorKind,
        utils::{self, constants::DEFAULT_SCHEMA_ATTRS, devsetup::SetupProfile},
    };

    // FUTURE - issuer and holder seperation only needed whilst modular deps not fully implemented
    async fn create_indy_proof(
        anoncreds_issuer: &Arc<dyn BaseAnonCreds>,
        anoncreds_holder: &Arc<dyn BaseAnonCreds>,
        ledger_read: &Arc<dyn AnoncredsLedgerRead>,
        ledger_write: &Arc<dyn AnoncredsLedgerWrite>,
        did: &str,
    ) -> (String, String, String, String) {
        let (schema_id, schema_json, cred_def_id, cred_def_json, cred_id) = create_and_store_nonrevocable_credential(
            anoncreds_issuer,
            anoncreds_holder,
            ledger_read,
            ledger_write,
            &did,
            DEFAULT_SCHEMA_ATTRS,
        )
        .await;
        let proof_req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "address1_1": json!({
                   "name":"address1",
                   "restrictions": [json!({ "issuer_did": did })]
               }),
               "zip_2": json!({
                   "name":"zip",
                   "restrictions": [json!({ "issuer_did": did })]
               }),
               "self_attest_3": json!({
                   "name":"self_attest",
               }),
           }),
           "requested_predicates": json!({}),
        })
        .to_string();
        let requested_credentials_json = json!({
              "self_attested_attributes":{
                 "self_attest_3": "my_self_attested_val"
              },
              "requested_attributes":{
                 "address1_1": {"cred_id": cred_id, "revealed": true},
                 "zip_2": {"cred_id": cred_id, "revealed": true}
                },
              "requested_predicates":{}
        })
        .to_string();

        let schema_json: serde_json::Value = serde_json::from_str(&schema_json).unwrap();
        let schemas = json!({
            schema_id: schema_json,
        })
        .to_string();

        let cred_def_json: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let cred_defs = json!({
            cred_def_id: cred_def_json,
        })
        .to_string();

        anoncreds_holder
            .prover_get_credentials_for_proof_req(&proof_req)
            .await
            .unwrap();

        let proof = anoncreds_holder
            .prover_create_proof(
                &proof_req,
                &requested_credentials_json,
                "main",
                &schemas,
                &cred_defs,
                None,
            )
            .await
            .unwrap();
        (schemas, cred_defs, proof_req, proof)
    }

    async fn create_proof_with_predicate(
        anoncreds_issuer: &Arc<dyn BaseAnonCreds>,
        anoncreds_holder: &Arc<dyn BaseAnonCreds>,
        ledger_read: &Arc<dyn AnoncredsLedgerRead>,
        ledger_write: &Arc<dyn AnoncredsLedgerWrite>,
        did: &str,
        include_predicate_cred: bool,
    ) -> (String, String, String, String) {
        let (schema_id, schema_json, cred_def_id, cred_def_json, cred_id) = create_and_store_nonrevocable_credential(
            anoncreds_issuer,
            anoncreds_holder,
            ledger_read,
            ledger_write,
            &did,
            DEFAULT_SCHEMA_ATTRS,
        )
        .await;

        let proof_req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "address1_1": json!({
                   "name":"address1",
                   "restrictions": [json!({ "issuer_did": did })]
               }),
               "self_attest_3": json!({
                   "name":"self_attest",
               }),
           }),
           "requested_predicates": json!({
               "zip_3": {"name":"zip", "p_type":">=", "p_value":18}
           }),
        })
        .to_string();

        let requested_credentials_json;
        if include_predicate_cred {
            requested_credentials_json = json!({
              "self_attested_attributes":{
                 "self_attest_3": "my_self_attested_val"
              },
              "requested_attributes":{
                 "address1_1": {"cred_id": cred_id, "revealed": true}
                },
              "requested_predicates":{
                  "zip_3": {"cred_id": cred_id}
              }
            })
            .to_string();
        } else {
            requested_credentials_json = json!({
              "self_attested_attributes":{
                 "self_attest_3": "my_self_attested_val"
              },
              "requested_attributes":{
                 "address1_1": {"cred_id": cred_id, "revealed": true}
                },
              "requested_predicates":{
              }
            })
            .to_string();
        }

        let schema_json: serde_json::Value = serde_json::from_str(&schema_json).unwrap();
        let schemas = json!({
            schema_id: schema_json,
        })
        .to_string();

        let cred_def_json: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let cred_defs = json!({
            cred_def_id: cred_def_json,
        })
        .to_string();

        anoncreds_holder
            .prover_get_credentials_for_proof_req(&proof_req)
            .await
            .unwrap();

        let proof = anoncreds_holder
            .prover_create_proof(
                &proof_req,
                &requested_credentials_json,
                "main",
                &schemas,
                &cred_defs,
                None,
            )
            .await
            .unwrap();
        (schemas, cred_defs, proof_req, proof)
    }

    async fn create_and_store_nonrevocable_credential(
        anoncreds_issuer: &Arc<dyn BaseAnonCreds>,
        anoncreds_holder: &Arc<dyn BaseAnonCreds>,
        ledger_read: &Arc<dyn AnoncredsLedgerRead>,
        ledger_write: &Arc<dyn AnoncredsLedgerWrite>,
        issuer_did: &str,
        attr_list: &str,
    ) -> (String, String, String, String, String) {
        let schema = create_and_write_test_schema(anoncreds_issuer, ledger_write, issuer_did, attr_list).await;

        let cred_def = create_and_write_test_cred_def(
            anoncreds_issuer,
            ledger_read,
            ledger_write,
            issuer_did,
            &schema.schema_id,
            false,
        )
        .await;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let cred_id = create_and_write_credential(
            anoncreds_issuer,
            anoncreds_holder,
            ledger_read,
            issuer_did,
            &cred_def,
            None,
        )
        .await;
        (
            schema.schema_id,
            schema.schema_json,
            cred_def.get_cred_def_id(),
            cred_def.get_cred_def_json().to_string(),
            cred_id,
        )
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_proof_self_attested_proof_validation() {
        SetupProfile::run(|setup| async move {
            let requested_attrs = json!([
                json!({
                    "name":"address1",
                    "self_attest_allowed": true,
                }),
                json!({
                    "name":"zip",
                    "self_attest_allowed": true,
                }),
            ])
            .to_string();
            let requested_predicates = json!([]).to_string();
            let revocation_details = r#"{"support_revocation":false}"#.to_string();
            let name = "Optional".to_owned();

            let proof_req_json = ProofRequestData::create(&setup.profile.inject_anoncreds(), &name)
                .await
                .unwrap()
                .set_requested_attributes_as_string(requested_attrs)
                .unwrap()
                .set_requested_predicates_as_string(requested_predicates)
                .unwrap()
                .set_not_revoked_interval(revocation_details)
                .unwrap();

            let proof_req_json = serde_json::to_string(&proof_req_json).unwrap();

            let anoncreds = Arc::clone(&setup.profile).inject_anoncreds();
            let prover_proof_json = anoncreds
                .prover_create_proof(
                    &proof_req_json,
                    &json!({
                      "self_attested_attributes":{
                         "attribute_0": "my_self_attested_address",
                         "attribute_1": "my_self_attested_zip"
                      },
                      "requested_attributes":{},
                      "requested_predicates":{}
                    })
                    .to_string(),
                    "main",
                    &json!({}).to_string(),
                    &json!({}).to_string(),
                    None,
                )
                .await
                .unwrap();

            assert_eq!(
                validate_indy_proof(
                    &setup.profile.inject_anoncreds_ledger_read(),
                    &setup.profile.inject_anoncreds(),
                    &prover_proof_json,
                    &proof_req_json
                )
                .await
                .unwrap(),
                true
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_proof_restrictions() {
        SetupProfile::run(|setup| async move {
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
            ])
            .to_string();
            let requested_predicates = json!([]).to_string();
            let revocation_details = r#"{"support_revocation":true}"#.to_string();
            let name = "Optional".to_owned();

            let proof_req_json = ProofRequestData::create(&setup.profile.inject_anoncreds(), &name)
                .await
                .unwrap()
                .set_requested_attributes_as_string(requested_attrs)
                .unwrap()
                .set_requested_predicates_as_string(requested_predicates)
                .unwrap()
                .set_not_revoked_interval(revocation_details)
                .unwrap();

            let proof_req_json = serde_json::to_string(&proof_req_json).unwrap();

            let (schema_id, schema_json, cred_def_id, cred_def_json, cred_id) =
                create_and_store_nonrevocable_credential(
                    &setup.profile.inject_anoncreds(),
                    &setup.profile.inject_anoncreds(),
                    &setup.profile.inject_anoncreds_ledger_read(),
                    &setup.profile.inject_anoncreds_ledger_write(),
                    &setup.institution_did,
                    utils::constants::DEFAULT_SCHEMA_ATTRS,
                )
                .await;
            let cred_def_json: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
            let schema_json: serde_json::Value = serde_json::from_str(&schema_json).unwrap();

            let anoncreds = Arc::clone(&setup.profile).inject_anoncreds();
            let prover_proof_json = anoncreds
                .prover_create_proof(
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
                    })
                    .to_string(),
                    "main",
                    &json!({ schema_id: schema_json }).to_string(),
                    &json!({ cred_def_id: cred_def_json }).to_string(),
                    None,
                )
                .await
                .unwrap();
            assert_eq!(
                validate_indy_proof(
                    &setup.profile.inject_anoncreds_ledger_read(),
                    &setup.profile.inject_anoncreds(),
                    &prover_proof_json,
                    &proof_req_json
                )
                .await
                .unwrap_err()
                .kind(),
                AriesVcxErrorKind::ProofRejected
            );

            let mut proof_req_json: serde_json::Value = serde_json::from_str(&proof_req_json).unwrap();
            proof_req_json["requested_attributes"]["attribute_0"]["restrictions"] = json!({});
            assert_eq!(
                validate_indy_proof(
                    &setup.profile.inject_anoncreds_ledger_read(),
                    &setup.profile.inject_anoncreds(),
                    &prover_proof_json,
                    &proof_req_json.to_string()
                )
                .await
                .unwrap(),
                true
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_proof_validate_attribute() {
        SetupProfile::run(|setup| async move {
            let requested_attrs = json!([
                json!({
                    "name":"address1",
                    "restrictions": [json!({ "issuer_did": setup.institution_did })]
                }),
                json!({
                    "name":"zip",
                    "restrictions": [json!({ "issuer_did": setup.institution_did })]
                }),
                json!({
                    "name":"self_attest",
                    "self_attest_allowed": true,
                }),
            ])
            .to_string();
            let requested_predicates = json!([]).to_string();
            let revocation_details = r#"{"support_revocation":true}"#.to_string();
            let name = "Optional".to_owned();

            let proof_req_json = ProofRequestData::create(&setup.profile.inject_anoncreds(), &name)
                .await
                .unwrap()
                .set_requested_attributes_as_string(requested_attrs)
                .unwrap()
                .set_requested_predicates_as_string(requested_predicates)
                .unwrap()
                .set_not_revoked_interval(revocation_details)
                .unwrap();

            let proof_req_json = serde_json::to_string(&proof_req_json).unwrap();

            let (schema_id, schema_json, cred_def_id, cred_def_json, cred_id) =
                create_and_store_nonrevocable_credential(
                    &setup.profile.inject_anoncreds(),
                    &setup.profile.inject_anoncreds(),
                    &setup.profile.inject_anoncreds_ledger_read(),
                    &setup.profile.inject_anoncreds_ledger_write(),
                    &setup.institution_did,
                    utils::constants::DEFAULT_SCHEMA_ATTRS,
                )
                .await;
            let cred_def_json: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
            let schema_json: serde_json::Value = serde_json::from_str(&schema_json).unwrap();

            let anoncreds = Arc::clone(&setup.profile).inject_anoncreds();
            let prover_proof_json = anoncreds
                .prover_create_proof(
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
                    })
                    .to_string(),
                    "main",
                    &json!({ schema_id: schema_json }).to_string(),
                    &json!({ cred_def_id: cred_def_json }).to_string(),
                    None,
                )
                .await
                .unwrap();
            assert_eq!(
                validate_indy_proof(
                    &setup.profile.inject_anoncreds_ledger_read(),
                    &setup.profile.inject_anoncreds(),
                    &prover_proof_json,
                    &proof_req_json
                )
                .await
                .unwrap(),
                true
            );

            let mut proof_obj: serde_json::Value = serde_json::from_str(&prover_proof_json).unwrap();
            {
                proof_obj["requested_proof"]["revealed_attrs"]["address1_1"]["raw"] = json!("Other Value");
                let prover_proof_json = serde_json::to_string(&proof_obj).unwrap();

                assert_eq!(
                    validate_indy_proof(
                        &setup.profile.inject_anoncreds_ledger_read(),
                        &setup.profile.inject_anoncreds(),
                        &prover_proof_json,
                        &proof_req_json
                    )
                    .await
                    .unwrap_err()
                    .kind(),
                    AriesVcxErrorKind::InvalidProof
                );
            }
            {
                proof_obj["requested_proof"]["revealed_attrs"]["address1_1"]["encoded"] =
                    json!("1111111111111111111111111111111111111111111111111111111111");
                let prover_proof_json = serde_json::to_string(&proof_obj).unwrap();

                assert_eq!(
                    validate_indy_proof(
                        &setup.profile.inject_anoncreds_ledger_read(),
                        &setup.profile.inject_anoncreds(),
                        &prover_proof_json,
                        &proof_req_json
                    )
                    .await
                    .unwrap_err()
                    .kind(),
                    AriesVcxErrorKind::InvalidProof
                );
            }
        })
        .await;
    }
    #[tokio::test]
    #[ignore]
    async fn test_pool_prover_verify_proof() {
        SetupProfile::run(|setup| async move {
            let (schemas, cred_defs, proof_req, proof) = create_indy_proof(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
            )
            .await;

            let anoncreds = Arc::clone(&setup.profile).inject_anoncreds();
            let proof_validation = anoncreds
                .verifier_verify_proof(&proof_req, &proof, &schemas, &cred_defs, "{}", "{}")
                .await
                .unwrap();

            assert!(proof_validation);
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_prover_verify_proof_with_predicate_success_case() {
        SetupProfile::run(|setup| async move {
            let (schemas, cred_defs, proof_req, proof) = create_proof_with_predicate(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
                true,
            )
            .await;

            let anoncreds = Arc::clone(&setup.profile).inject_anoncreds();
            let proof_validation = anoncreds
                .verifier_verify_proof(&proof_req, &proof, &schemas, &cred_defs, "{}", "{}")
                .await
                .unwrap();

            assert!(proof_validation);
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_prover_verify_proof_with_predicate_fail_case() {
        SetupProfile::run(|setup| async move {
            let (schemas, cred_defs, proof_req, proof) = create_proof_with_predicate(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
                false,
            )
            .await;

            let anoncreds = Arc::clone(&setup.profile).inject_anoncreds();
            anoncreds
                .verifier_verify_proof(&proof_req, &proof, &schemas, &cred_defs, "{}", "{}")
                .await
                .unwrap_err();
        })
        .await;
    }
}
