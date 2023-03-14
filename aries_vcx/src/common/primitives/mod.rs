pub mod credential_definition;
pub mod credential_schema;
pub mod revocation_registry;
pub mod revocation_registry_delta;

#[cfg(test)]
#[cfg(feature = "pool_tests")]
pub mod integration_tests {
    use std::sync::Arc;

    use crate::{
        common::{
            primitives::revocation_registry::generate_rev_reg,
            test_utils::{
                create_and_store_credential_def, create_and_store_nonrevocable_credential_def,
                create_and_write_test_schema,
            },
        },
        errors::error::AriesVcxErrorKind,
        utils::{constants::DEFAULT_SCHEMA_ATTRS, devsetup::SetupProfile, get_temp_dir_path},
    };

    #[tokio::test]
    async fn test_rev_reg_def_fails_for_cred_def_created_without_revocation() {
        // todo: does not need agency setup
        SetupProfile::run_indy(|setup| async move {
            // Cred def is created with support_revocation=false,
            // revoc_reg_def will fail in libindy because cred_Def doesn't have revocation keys
            let (_, _, cred_def_id, _, _) = create_and_store_nonrevocable_credential_def(
                &setup.profile,
                &setup.institution_did,
                DEFAULT_SCHEMA_ATTRS,
            )
            .await;

            let rc = generate_rev_reg(
                &setup.profile,
                &setup.institution_did,
                &cred_def_id,
                get_temp_dir_path("path.txt").to_str().unwrap(),
                2,
                "tag1",
            )
            .await;

            assert_eq!(rc.unwrap_err().kind(), AriesVcxErrorKind::LibindyInvalidStructure);
        })
        .await;
    }

    #[tokio::test]
    async fn test_get_rev_reg_def_json() {
        SetupProfile::run_indy(|setup| async move {
            let attrs = r#"["address1","address2","city","state","zip"]"#;
            let (_, _, _, _, rev_reg_id, _, _) =
                create_and_store_credential_def(&setup.profile, &setup.institution_did, attrs).await;

            let ledger = Arc::clone(&setup.profile).inject_ledger();
            let _json = ledger.get_rev_reg_def_json(&rev_reg_id).await.unwrap();
        })
        .await;
    }

    #[tokio::test]
    async fn test_get_rev_reg_delta_json() {
        SetupProfile::run_indy(|setup| async move {
            let attrs = r#"["address1","address2","city","state","zip"]"#;
            let (_, _, _, _, rev_reg_id, _, _) =
                create_and_store_credential_def(&setup.profile, &setup.institution_did, attrs).await;

            let ledger = Arc::clone(&setup.profile).inject_ledger();
            let (id, _delta, _timestamp) = ledger.get_rev_reg_delta_json(&rev_reg_id, None, None).await.unwrap();

            assert_eq!(id, rev_reg_id);
        })
        .await;
    }

    #[tokio::test]
    async fn test_get_rev_reg() {
        SetupProfile::run_indy(|setup| async move {
            let attrs = r#"["address1","address2","city","state","zip"]"#;
            let (_, _, _, _, rev_reg_id, _, _) =
                create_and_store_credential_def(&setup.profile, &setup.institution_did, attrs).await;

            let ledger = Arc::clone(&setup.profile).inject_ledger();
            let (id, _rev_reg, _timestamp) = ledger
                .get_rev_reg(&rev_reg_id, time::get_time().sec as u64)
                .await
                .unwrap();

            assert_eq!(id, rev_reg_id);
        })
        .await;
    }

    #[tokio::test]
    async fn test_get_cred_def() {
        SetupProfile::run_indy(|setup| async move {
            let attrs = r#"["address1","address2","city","state","zip"]"#;
            let (_, _, cred_def_id, cred_def_json, _) =
                create_and_store_nonrevocable_credential_def(&setup.profile, &setup.institution_did, attrs).await;

            let ledger = Arc::clone(&setup.profile).inject_ledger();
            let cred_def = ledger.get_cred_def(&cred_def_id, None).await.unwrap();

            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&cred_def).unwrap(),
                serde_json::from_str::<serde_json::Value>(&cred_def_json).unwrap()
            );
        })
        .await;
    }

    #[tokio::test]
    async fn from_pool_ledger_with_id() {
        SetupProfile::run_indy(|setup| async move {
            let (schema_id, _schema_json) =
                create_and_write_test_schema(&setup.profile, &setup.institution_did, DEFAULT_SCHEMA_ATTRS).await;

            let ledger = Arc::clone(&setup.profile).inject_ledger();
            let rc = ledger.get_schema(&schema_id, None).await;

            let retrieved_schema = rc.unwrap();
            assert!(retrieved_schema.contains(&schema_id));
        })
        .await;
    }
}
