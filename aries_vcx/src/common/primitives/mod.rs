pub mod credential_definition;
pub mod credential_schema;
pub mod revocation_registry;
pub mod revocation_registry_delta;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub mod integration_tests {
    use aries_vcx_core::ledger::indy::pool::test_utils::get_temp_file_path;
    use std::sync::Arc;

    use crate::common::primitives::revocation_registry::generate_rev_reg;
    use crate::common::test_utils::{
        create_and_store_credential_def_and_rev_reg, create_and_store_nonrevocable_credential_def,
        create_and_write_test_schema,
    };
    use crate::errors::error::AriesVcxErrorKind;
    use crate::utils::constants::DEFAULT_SCHEMA_ATTRS;
    use crate::utils::devsetup::SetupProfile;

    #[tokio::test]
    #[ignore]
    async fn test_pool_rev_reg_def_fails_for_cred_def_created_without_revocation() {
        SetupProfile::run(|setup| async move {
            // Cred def is created with support_revocation=false,
            // revoc_reg_def will fail in libindy because cred_Def doesn't have revocation keys
            let (_, _, cred_def_id, _, _) = create_and_store_nonrevocable_credential_def(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
                DEFAULT_SCHEMA_ATTRS,
            )
            .await;

            let rc = generate_rev_reg(
                &setup.profile.inject_anoncreds(),
                &setup.institution_did,
                &cred_def_id,
                get_temp_file_path("path.txt").to_str().unwrap(),
                2,
                "tag1",
            )
            .await;

            #[cfg(feature = "modular_libs")]
            assert_eq!(rc.unwrap_err().kind(), AriesVcxErrorKind::InvalidState);
            #[cfg(not(feature = "modular_libs"))]
            assert_eq!(rc.unwrap_err().kind(), AriesVcxErrorKind::InvalidInput);
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_get_rev_reg_def_json() {
        SetupProfile::run(|setup| async move {
            let attrs = r#"["address1","address2","city","state","zip"]"#;
            let (_, _, _, _, rev_reg_id, _, _, _) = create_and_store_credential_def_and_rev_reg(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
                attrs,
            )
            .await;

            let ledger = Arc::clone(&setup.profile).inject_anoncreds_ledger_read();
            let _json = ledger.get_rev_reg_def_json(&rev_reg_id).await.unwrap();
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_get_rev_reg_delta_json() {
        SetupProfile::run(|setup| async move {
            let attrs = r#"["address1","address2","city","state","zip"]"#;
            let (_, _, _, _, rev_reg_id, _, _, _) = create_and_store_credential_def_and_rev_reg(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
                attrs,
            )
            .await;

            let ledger = Arc::clone(&setup.profile).inject_anoncreds_ledger_read();
            let (id, _delta, _timestamp) = ledger.get_rev_reg_delta_json(&rev_reg_id, None, None).await.unwrap();

            assert_eq!(id, rev_reg_id);
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_get_rev_reg() {
        SetupProfile::run(|setup| async move {
            let attrs = r#"["address1","address2","city","state","zip"]"#;
            let (_, _, _, _, rev_reg_id, _, _, _) = create_and_store_credential_def_and_rev_reg(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
                attrs,
            )
            .await;

            let ledger = Arc::clone(&setup.profile).inject_anoncreds_ledger_read();
            let (id, _rev_reg, _timestamp) = ledger
                .get_rev_reg(&rev_reg_id, time::OffsetDateTime::now_utc().unix_timestamp() as u64)
                .await
                .unwrap();

            assert_eq!(id, rev_reg_id);
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_get_cred_def() {
        SetupProfile::run(|setup| async move {
            let attrs = r#"["address1","address2","city","state","zip"]"#;
            let (_, _, cred_def_id, cred_def_json, _) = create_and_store_nonrevocable_credential_def(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_read(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
                attrs,
            )
            .await;

            let ledger = Arc::clone(&setup.profile).inject_anoncreds_ledger_read();
            let cred_def = ledger.get_cred_def(&cred_def_id, None).await.unwrap();

            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&cred_def).unwrap(),
                serde_json::from_str::<serde_json::Value>(&cred_def_json).unwrap()
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_pool_create_and_get_schema() {
        SetupProfile::run(|setup| async move {
            let (schema_id, _schema_json) = create_and_write_test_schema(
                &setup.profile.inject_anoncreds(),
                &setup.profile.inject_anoncreds_ledger_write(),
                &setup.institution_did,
                DEFAULT_SCHEMA_ATTRS,
            )
            .await;

            let ledger = Arc::clone(&setup.profile).inject_anoncreds_ledger_read();
            let rc = ledger.get_schema(&schema_id, None).await;

            let retrieved_schema = rc.unwrap();
            assert!(retrieved_schema.contains(&schema_id));
        })
        .await;
    }
}
