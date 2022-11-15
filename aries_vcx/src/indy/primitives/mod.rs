pub mod revocation_registry;
pub mod credential_definition;
pub mod credential_schema;

#[cfg(test)]
#[cfg(feature = "pool_tests")]
pub mod integration_tests {
    use crate::error::VcxErrorKind;
    use crate::indy::test_utils::{
        create_and_store_credential_def, create_and_store_nonrevocable_credential_def,
        create_and_write_test_schema,
    };
    use crate::indy::ledger::transactions::{
        get_cred_def, get_rev_reg, get_rev_reg_def_json, get_rev_reg_delta_json,
        get_schema_json, is_cred_def_on_ledger,
    };
    use crate::indy::primitives::revocation_registry::generate_rev_reg;
    use crate::utils::constants::DEFAULT_SCHEMA_ATTRS;
    use crate::utils::devsetup::SetupIndyWalletPool;
    use crate::utils::get_temp_dir_path;

    #[tokio::test]
    async fn test_rev_reg_def_fails_for_cred_def_created_without_revocation() {
        // todo: does not need agency setup
        let setup = SetupWalletPool::init().await;

        // Cred def is created with support_revocation=false,
        // revoc_reg_def will fail in libindy because cred_Def doesn't have revocation keys
        let (_, _, cred_def_id, _, _) =
            create_and_store_nonrevocable_credential_def(
                setup.wallet_handle,
                setup.pool_handle,
                &setup.institution_did, DEFAULT_SCHEMA_ATTRS)
            .await;

        let rc =
            generate_rev_reg(
                setup.wallet_handle,
                &setup.institution_did,
                &cred_def_id,
                get_temp_dir_path("path.txt").to_str().unwrap(),
                2,
                "tag1")
            .await;

        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::LibindyInvalidStructure);
    }

    #[tokio::test]
    async fn test_get_rev_reg_def_json() {
        let setup = SetupWalletPool::init().await;

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (_, _, _, _, rev_reg_id, _, _) =
            create_and_store_credential_def(
                setup.wallet_handle,
                setup.pool_handle,
                &setup.institution_did,
                attrs)
            .await;

        let (id, _json) = get_rev_reg_def_json(setup.pool_handle, &rev_reg_id).await.unwrap();

        assert_eq!(id, rev_reg_id);
    }

    #[tokio::test]
    async fn test_get_rev_reg_delta_json() {
        let setup = SetupWalletPool::init().await;

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (_, _, _, _, rev_reg_id, _, _) =
            create_and_store_credential_def(
                setup.wallet_handle,
                setup.pool_handle,
                &setup.institution_did,
                attrs)
            .await;

        let (id, _delta, _timestamp) =
            get_rev_reg_delta_json(
                setup.pool_handle,
                &rev_reg_id, None, None)
            .await.unwrap();

        assert_eq!(id, rev_reg_id);
    }

    #[tokio::test]
    async fn test_get_rev_reg() {
        let setup = SetupWalletPool::init().await;

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (_, _, _, _, rev_reg_id, _, _) =
            create_and_store_credential_def(
                setup.wallet_handle,
                setup.pool_handle,
                &setup.institution_did, attrs)
            .await;

        let (id, _rev_reg, _timestamp) =
            get_rev_reg(
                setup.pool_handle,
                &rev_reg_id,
                time::get_time().sec as u64)
            .await.unwrap();

        assert_eq!(id, rev_reg_id);
    }

    #[tokio::test]
    async fn test_get_cred_def() {
        let setup = SetupWalletPool::init().await;

        let attrs = r#"["address1","address2","city","state","zip"]"#;
        let (_, _, cred_def_id, cred_def_json, _) =
            create_and_store_nonrevocable_credential_def(
                setup.wallet_handle,
                setup.pool_handle,
                &setup.institution_did,
                attrs)
            .await;

        let (id, cred_def) =
            get_cred_def(
                setup.pool_handle,
                None,
                &cred_def_id)
            .await
            .unwrap();

        assert_eq!(id, cred_def_id);
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&cred_def).unwrap(),
            serde_json::from_str::<serde_json::Value>(&cred_def_json).unwrap()
        );
    }

    #[tokio::test]
    async fn test_is_cred_def_on_ledger() {
        let setup = SetupWalletPool::init().await;

        assert_eq!(
            is_cred_def_on_ledger(setup.pool_handle, None, "V4SGRU86Z58d6TV7PBUe6f:3:CL:194:tag7")
                .await
                .unwrap(),
            false
        );
    }

    #[tokio::test]
    async fn from_pool_ledger_with_id() {
        let setup = SetupWalletPool::init().await;

        let (schema_id, _schema_json) =
            create_and_write_test_schema(
                setup.wallet_handle,
                setup.pool_handle,
                &setup.institution_did,
                DEFAULT_SCHEMA_ATTRS)
            .await;

        let rc = get_schema_json(setup.wallet_handle, setup.pool_handle, &schema_id).await;

        let (_id, retrieved_schema) = rc.unwrap();
        assert!(retrieved_schema.contains(&schema_id));
    }
}
