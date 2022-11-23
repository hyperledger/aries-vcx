pub mod encoding;
pub mod holder;
pub mod issuer;

use std::collections::HashMap;

use time::get_time;
use vdrtools::{PoolHandle, WalletHandle};

use crate::error::prelude::*;

use self::holder::libindy_prover_get_credential;

use super::primitives::revocation_registry_delta::RevocationRegistryDelta;

#[derive(Serialize, Deserialize)]
struct ProverCredential {
    referent: String,
    attrs: HashMap<String, String>,
    schema_id: String,
    cred_def_id: String,
    rev_reg_id: Option<String>,
    cred_rev_id: Option<String>,
}

pub async fn get_cred_rev_id(wallet_handle: WalletHandle, cred_id: &str) -> VcxResult<String> {
    let cred_json = libindy_prover_get_credential(wallet_handle, cred_id).await?;
    let prover_cred = serde_json::from_str::<ProverCredential>(&cred_json).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!("Failed to deserialize anoncreds credential: {}", err),
        )
    })?;
    prover_cred.cred_rev_id.ok_or(VcxError::from_msg(
        VcxErrorKind::InvalidRevocationDetails,
        "Credenial revocation id missing on credential - is this credential revokable?",
    ))
}

pub async fn is_cred_revoked(
    pool_handle: PoolHandle,
    rev_reg_id: &str,
    rev_id: &str,
) -> VcxResult<bool> {
    let from = None;
    let to = Some(get_time().sec as u64 + 100);
    let rev_reg_delta = RevocationRegistryDelta::create_from_ledger(pool_handle, rev_reg_id, from, to).await?;
    Ok(rev_reg_delta.revoked().iter().any(|s| s.to_string().eq(rev_id)))
}

#[cfg(test)]
#[cfg(feature = "pool_tests")]
mod integration_tests {
    use super::*;

    use crate::indy::primitives::revocation_registry::{self, publish_local_revocations};
    use crate::indy::test_utils::create_and_store_credential;
    use crate::utils::constants::DEFAULT_SCHEMA_ATTRS;
    use crate::utils::devsetup::SetupWalletPool;

    #[tokio::test]
    async fn test_prover_get_credential() {
        SetupWalletPool::run(|setup| async move {

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
        }).await;
    }

    #[tokio::test]
    async fn test_get_cred_rev_id() {
        SetupWalletPool::run(|setup| async move {

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
        }).await;
    }

    #[tokio::test]
    async fn test_is_cred_revoked() {
        SetupWalletPool::run(|setup| async move {

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
        }).await;
    }
}
