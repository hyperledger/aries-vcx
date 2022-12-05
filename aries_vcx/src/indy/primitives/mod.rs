pub mod credential_definition;
pub mod credential_schema;
pub mod revocation_registry;

#[cfg(test)]
#[cfg(feature = "pool_tests")]
pub mod integration_tests {
    use crate::{indy::ledger::transactions::is_cred_def_on_ledger, utils::devsetup::SetupWalletPool};

    #[tokio::test]
    async fn test_is_cred_def_on_ledger() {
        SetupWalletPool::run(|setup| async move {

        assert_eq!(
            is_cred_def_on_ledger(setup.pool_handle, None, "V4SGRU86Z58d6TV7PBUe6f:3:CL:194:tag7")
                .await
                .unwrap(),
            false
        );
        }).await;
    }
}
