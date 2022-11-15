pub mod holder;
pub mod issuer;

#[cfg(test)]
#[cfg(feature = "pool_tests")]
mod integration_tests {
    use super::*;

    use crate::indy::primitives::revocation_registry::{self, publish_local_revocations};
    use crate::indy::test_utils::create_and_store_credential;
    use crate::utils::constants::DEFAULT_SCHEMA_ATTRS;
    use crate::utils::devsetup::SetupIndyWalletPool;

    #[tokio::test]
    async fn test_prover_get_credential() {
        let setup = SetupWalletPool::init().await;

        let res = create_and_store_credential(
            setup.wallet_handle,
            setup.pool_handle,
            &setup.institution_did,
            DEFAULT_SCHEMA_ATTRS,
        )
        .await;
        let schema_id = res.0;
        let cred_def_id = res.2;
        let cred_id = res.7;
        let rev_reg_id = res.8;
        let cred_rev_id = res.9;

        let cred_json = libindy_prover_get_credential(setup.wallet_handle, &cred_id)
            .await
            .unwrap();
        let prover_cred = serde_json::from_str::<ProverCredential>(&cred_json).unwrap();

        assert_eq!(prover_cred.schema_id, schema_id);
        assert_eq!(prover_cred.cred_def_id, cred_def_id);
        assert_eq!(prover_cred.cred_rev_id.unwrap().to_string(), cred_rev_id);
        assert_eq!(prover_cred.rev_reg_id.unwrap(), rev_reg_id);
    }

    #[tokio::test]
    async fn test_get_cred_rev_id() {
        let setup = SetupWalletPool::init().await;

        let res = create_and_store_credential(
            setup.wallet_handle,
            setup.pool_handle,
            &setup.institution_did,
            DEFAULT_SCHEMA_ATTRS,
        )
        .await;
        let cred_id = res.7;
        let cred_rev_id = res.9;

        let cred_rev_id_ = get_cred_rev_id(setup.wallet_handle, &cred_id).await.unwrap();

        assert_eq!(cred_rev_id, cred_rev_id_.to_string());
    }

    #[tokio::test]
    async fn test_is_cred_revoked() {
        let setup = SetupWalletPool::init().await;

        let res = create_and_store_credential(
            setup.wallet_handle,
            setup.pool_handle,
            &setup.institution_did,
            DEFAULT_SCHEMA_ATTRS,
        )
        .await;
        let rev_reg_id = res.8;
        let cred_rev_id = res.9;
        let tails_file = res.10;

        assert!(
            !is_cred_revoked(setup.pool_handle, &rev_reg_id, &cred_rev_id)
                .await
                .unwrap()
        );

        revocation_registry::revoke_credential_local(setup.wallet_handle, &tails_file, &rev_reg_id, &cred_rev_id)
            .await
            .unwrap();
        publish_local_revocations(
            setup.wallet_handle,
            setup.pool_handle,
            &setup.institution_did,
            &rev_reg_id,
        )
        .await
        .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(500));

        assert!(
            is_cred_revoked(setup.pool_handle, &rev_reg_id, &cred_rev_id)
                .await
                .unwrap()
        );
    }
}
