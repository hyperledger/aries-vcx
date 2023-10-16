// TODO: Convert to unit test or move to ./tests
#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod integration_tests {
    use aries_vcx_core::{
        errors::error::AriesVcxCoreErrorKind,
        ledger::{base_ledger::AnoncredsLedgerRead, indy::pool::test_utils::get_temp_dir_path},
    };

    use crate::common::{
        credentials::get_cred_rev_id,
        test_utils::{
            create_and_publish_test_rev_reg, create_and_write_credential,
            create_and_write_test_cred_def, create_and_write_test_schema,
        },
    };

    #[tokio::test]
    #[ignore]
    async fn test_pool_returns_error_if_proof_request_is_malformed() {
        run_setup!(|setup| async move {
            let proof_req = "{";
            let anoncreds = setup.anoncreds;
            let result = anoncreds
                .prover_get_credentials_for_proof_req(&setup.wallet, proof_req)
                .await;
            assert_eq!(
                result.unwrap_err().kind(),
                AriesVcxCoreErrorKind::InvalidProofRequest
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_prover_get_credentials() {
        run_setup!(|setup| async move {
            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({
                   "address1_1": json!({
                       "name":"address1",
                   }),
                   "zip_2": json!({
                       "name":"zip",
                   }),
               }),
               "requested_predicates": json!({}),
            })
            .to_string();

            let anoncreds = setup.anoncreds;
            let _result = anoncreds
                .prover_get_credentials_for_proof_req(&setup.wallet, &proof_req)
                .await
                .unwrap();

            let result_malformed_json = anoncreds
                .prover_get_credentials_for_proof_req(&setup.wallet, "{}")
                .await
                .unwrap_err();
            assert_eq!(
                result_malformed_json.kind(),
                AriesVcxCoreErrorKind::InvalidAttributesStructure
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_proof_req_attribute_names() {
        run_setup!(|setup| async move {
            let proof_req = json!({
               "nonce":"123432421212",
               "name":"proof_req_1",
               "version":"0.1",
               "requested_attributes": json!({
                   "multiple_attrs": {
                       "names": ["name_1", "name_2"]
                   },
                   "address1_1": json!({
                       "name":"address1",
                       "restrictions": [json!({ "issuer_did": "some_did" })]
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

            let anoncreds = setup.anoncreds;
            let _result = anoncreds
                .prover_get_credentials_for_proof_req(&setup.wallet, &proof_req)
                .await
                .unwrap();
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_revoke_credential() {
        run_setup!(|setup| async move {
            let schema = create_and_write_test_schema(
                &setup.wallet,
                &setup.anoncreds,
                &setup.ledger_write,
                &setup.institution_did,
                crate::utils::constants::DEFAULT_SCHEMA_ATTRS,
            )
            .await;
            let cred_def = create_and_write_test_cred_def(
                &setup.wallet,
                &setup.anoncreds,
                &setup.ledger_read,
                &setup.ledger_write,
                &setup.institution_did,
                &schema.schema_id,
                true,
            )
            .await;
            let rev_reg = create_and_publish_test_rev_reg(
                &setup.wallet,
                &setup.anoncreds,
                &setup.ledger_write,
                &setup.institution_did,
                &cred_def.get_cred_def_id(),
            )
            .await;
            let cred_id = create_and_write_credential(
                &setup.wallet,
                &setup.wallet,
                &setup.anoncreds,
                &setup.anoncreds,
                &setup.institution_did,
                &cred_def,
                Some(&rev_reg),
            )
            .await;
            let cred_rev_id = get_cred_rev_id(&setup.wallet, &setup.anoncreds, &cred_id)
                .await
                .unwrap();

            let ledger = setup.ledger_read;

            let (_, first_rev_reg_delta, first_timestamp) = ledger
                .get_rev_reg_delta_json(&rev_reg.rev_reg_id, None, None)
                .await
                .unwrap();

            let (_, test_same_delta, test_same_timestamp) = ledger
                .get_rev_reg_delta_json(&rev_reg.rev_reg_id, None, None)
                .await
                .unwrap();

            assert_eq!(first_rev_reg_delta, test_same_delta);
            assert_eq!(first_timestamp, test_same_timestamp);

            let anoncreds = &setup.anoncreds;

            anoncreds
                .revoke_credential_local(
                    &setup.wallet,
                    get_temp_dir_path().to_str().unwrap(),
                    &rev_reg.rev_reg_id,
                    &cred_rev_id,
                )
                .await
                .unwrap();

            rev_reg
                .publish_local_revocations(
                    &setup.wallet,
                    &setup.anoncreds,
                    &setup.ledger_write,
                    &setup.institution_did,
                )
                .await
                .unwrap();

            // Delta should change after revocation
            let (_, second_rev_reg_delta, _) = ledger
                .get_rev_reg_delta_json(&rev_reg.rev_reg_id, Some(first_timestamp + 1), None)
                .await
                .unwrap();

            assert_ne!(first_rev_reg_delta, second_rev_reg_delta);
        })
        .await;
    }
}
