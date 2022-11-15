use vdrtools_sys::{PoolHandle, WalletHandle};

use crate::error::{VcxError, VcxResult};
use crate::global::settings;
use crate::indy::ledger::transactions::{
    build_cred_def_request, check_response,sign_and_submit_to_ledger
};

// consider relocating
pub async fn publish_cred_def(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    issuer_did: &str,
    cred_def_json: &str,
) -> VcxResult<()> {
    trace!(
        "publish_cred_def >>> issuer_did: {}, cred_def_json: {}",
        issuer_did,
        cred_def_json
    );
    if settings::indy_mocks_enabled() {
        debug!("publish_cred_def >>> mocked success");
        return Ok(());
    }
    let cred_def_req = build_cred_def_request(issuer_did, cred_def_json).await?;
    let response = sign_and_submit_to_ledger(wallet_handle, pool_handle, issuer_did, &cred_def_req).await?;
    check_response(&response)
}

// consider relocating
pub async fn libindy_create_and_store_credential_def(
    wallet_handle: WalletHandle,
    issuer_did: &str,
    schema_json: &str,
    tag: &str,
    sig_type: Option<&str>,
    config_json: &str,
) -> VcxResult<(String, String)> {
    vdrtools::anoncreds::issuer_create_and_store_credential_def(
        wallet_handle,
        issuer_did,
        schema_json,
        tag,
        sig_type,
        config_json,
    )
    .await
    .map_err(VcxError::from)
}

#[cfg(test)]
#[cfg(feature = "pool_tests")]
pub mod integration_tests {
    use crate::indy::ledger::transactions::get_schema_json;
    use crate::indy::primitives::credential_definition::generate_cred_def;
    use crate::indy::primitives::credential_definition::publish_cred_def;
    use crate::indy::primitives::revocation_registry::{generate_rev_reg, publish_rev_reg_def, publish_rev_reg_delta};
    use crate::indy::test_utils::create_and_write_test_schema;
    use crate::utils::constants::DEFAULT_SCHEMA_ATTRS;
    use crate::utils::devsetup::SetupIndyWalletPool;

    #[tokio::test]
    async fn test_create_cred_def_real() {
        let setup = SetupWalletPool::init().await;

        let (schema_id, _) = create_and_write_test_schema(
            setup.wallet_handle,
            setup.pool_handle,
            &setup.institution_did,
            DEFAULT_SCHEMA_ATTRS,
        )
        .await;

        let (_, schema_json) = get_schema_json(setup.wallet_handle, setup.pool_handle, &schema_id)
            .await
            .unwrap();

        let (_, cred_def_json) = generate_cred_def(
            setup.wallet_handle,
            &setup.institution_did,
            &schema_json,
            "tag_1",
            None,
            Some(true),
        )
        .await
        .unwrap();

        publish_cred_def(
            setup.wallet_handle,
            setup.pool_handle,
            &setup.institution_did,
            &cred_def_json,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_create_rev_reg_def() {
        let setup = SetupWalletPool::init().await;

        let (schema_id, _) = create_and_write_test_schema(
            setup.wallet_handle,
            setup.pool_handle,
            &setup.institution_did,
            DEFAULT_SCHEMA_ATTRS,
        )
        .await;
        let (_, schema_json) = get_schema_json(setup.wallet_handle, setup.pool_handle, &schema_id)
            .await
            .unwrap();

        let (cred_def_id, cred_def_json) = generate_cred_def(
            setup.wallet_handle,
            &setup.institution_did,
            &schema_json,
            "tag_1",
            None,
            Some(true),
        )
        .await
        .unwrap();
        publish_cred_def(
            setup.wallet_handle,
            setup.pool_handle,
            &setup.institution_did,
            &cred_def_json,
        )
        .await
        .unwrap();
        let (rev_reg_def_id, rev_reg_def_json, rev_reg_entry_json) = generate_rev_reg(
            setup.wallet_handle,
            &setup.institution_did,
            &cred_def_id,
            "tails.txt",
            2,
            "tag1",
        )
        .await
        .unwrap();
        publish_rev_reg_def(
            setup.wallet_handle,
            setup.pool_handle,
            &setup.institution_did,
            &rev_reg_def_json,
        )
        .await
        .unwrap();
        publish_rev_reg_delta(
            setup.wallet_handle,
            setup.pool_handle,
            &setup.institution_did,
            &rev_reg_def_id,
            &rev_reg_entry_json,
        )
        .await
        .unwrap();
    }
}
