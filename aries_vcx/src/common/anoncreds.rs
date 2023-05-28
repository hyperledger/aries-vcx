#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod integration_tests {
    use std::sync::Arc;

    use aries_vcx_core::errors::error::AriesVcxCoreErrorKind;

    use crate::common::test_utils::create_and_store_credential;
    use crate::utils::constants::TAILS_DIR;
    use crate::utils::devsetup::SetupProfile;
    use crate::utils::get_temp_dir_path;

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

    #[tokio::test]
    #[ignore]
    async fn test_pool_revoke_credential() {
        SetupProfile::run(|setup| async move {
            let (_, _, _, _, _, _, _, _, rev_reg_id, cred_rev_id, _, rev_reg) = create_and_store_credential(
                &setup.profile,
                &setup.profile,
                &setup.institution_did,
                crate::utils::constants::DEFAULT_SCHEMA_ATTRS,
            )
            .await;

            let ledger = Arc::clone(&setup.profile).inject_anoncreds_ledger_read();

            let (_, first_rev_reg_delta, first_timestamp) =
                ledger.get_rev_reg_delta_json(&rev_reg_id, None, None).await.unwrap();

            let (_, test_same_delta, test_same_timestamp) =
                ledger.get_rev_reg_delta_json(&rev_reg_id, None, None).await.unwrap();

            assert_eq!(first_rev_reg_delta, test_same_delta);
            assert_eq!(first_timestamp, test_same_timestamp);

            let anoncreds = Arc::clone(&setup.profile).inject_anoncreds();

            anoncreds
                .revoke_credential_local(
                    get_temp_dir_path(TAILS_DIR).to_str().unwrap(),
                    &rev_reg_id,
                    &cred_rev_id,
                )
                .await
                .unwrap();

            rev_reg
                .publish_local_revocations(&setup.profile, &setup.institution_did)
                .await
                .unwrap();

            // Delta should change after revocation
            let (_, second_rev_reg_delta, _) = ledger
                .get_rev_reg_delta_json(&rev_reg_id, Some(first_timestamp + 1), None)
                .await
                .unwrap();

            assert_ne!(first_rev_reg_delta, second_rev_reg_delta);
        })
        .await;
    }
}
