// TODO: Convert to unit test or move to ./tests
#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod integration_tests {
    use std::sync::Arc;

    use aries_vcx_core::errors::error::AriesVcxCoreErrorKind;
    use aries_vcx_core::ledger::indy::pool::test_utils::get_temp_dir_path;

    use crate::common::credentials::get_cred_rev_id;
    use crate::common::test_utils::{
        create_and_write_credential, create_and_write_test_cred_def, create_and_write_test_rev_reg,
        create_and_write_test_schema,
    };
    use crate::utils::devsetup::SetupProfile;

    #[tokio::test]
    #[ignore]
    async fn test_pool_returns_error_if_proof_request_is_malformed() {
        SetupProfile::run(|setup| async move {
            let proof_req = "{";
            let anoncreds = Arc::clone(&setup.profile).inject_anoncreds();
            let result = anoncreds.prover_get_credentials_for_proof_req(&proof_req).await;
            assert_eq!(result.unwrap_err().kind(), AriesVcxCoreErrorKind::InvalidProofRequest);
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_prover_get_credentials() {
        SetupProfile::run(|setup| async move {
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

            let anoncreds = Arc::clone(&setup.profile).inject_anoncreds();
            let _result = anoncreds
                .prover_get_credentials_for_proof_req(&proof_req)
                .await
                .unwrap();

            let result_malformed_json = anoncreds.prover_get_credentials_for_proof_req("{}").await.unwrap_err();
            assert_eq!(
                result_malformed_json.kind(),
                AriesVcxCoreErrorKind::InvalidAttributesStructure
            );
        })
        .await;
    }

    // #[cfg(feature = "modular_libs")]
    #[tokio::test]
    #[ignore]
    async fn test_pool_proof_req_attribute_names() {
        SetupProfile::run(|setup| async move {
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

            let anoncreds = Arc::clone(&setup.profile).inject_anoncreds();
            let _result = anoncreds
                .prover_get_credentials_for_proof_req(&proof_req)
                .await
                .unwrap();
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_revoke_credential() {
        SetupProfile::run(|setup| async move {
            let schema = create_and_write_test_schema(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
                crate::utils::constants::DEFAULT_SCHEMA_ATTRS,
            )
            .await;
            let cred_def = create_and_write_test_cred_def(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
                &schema.schema_id,
            )
            .await;
            let rev_reg = create_and_write_test_rev_reg(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
                &cred_def.get_cred_def_id(),
            )
            .await;
            let cred_id = create_and_write_credential(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.institution_did,
                &rev_reg,
                &cred_def,
            )
            .await;
            let cred_rev_id = get_cred_rev_id(&setup.profile.inject_anoncreds(), &cred_id)
                .await
                .unwrap();

            let ledger = Arc::clone(&setup.profile).inject_anoncreds_ledger_read();

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

            let anoncreds = Arc::clone(&setup.profile).inject_anoncreds();

            anoncreds
                .revoke_credential_local(get_temp_dir_path().to_str().unwrap(), &rev_reg.rev_reg_id, &cred_rev_id)
                .await
                .unwrap();

            rev_reg
                .publish_local_revocations(
                    &setup.profile.inject_anoncreds(),
                    &setup.profile.inject_anoncreds_ledger_write(),
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
