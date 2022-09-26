use vdrtools_sys::WalletHandle;
use crate::error::VcxResult;
use crate::global::settings;
use crate::indy::anoncreds;
use crate::utils::constants::{CRED_DEF_ID, CRED_DEF_JSON};

pub mod creddef_libindy;

pub async fn generate_cred_def(
    wallet_handle: WalletHandle,
    issuer_did: &str,
    schema_json: &str,
    tag: &str,
    sig_type: Option<&str>,
    support_revocation: Option<bool>,
) -> VcxResult<(String, String)> {
    trace!(
        "generate_cred_def >>> issuer_did: {}, schema_json: {}, tag: {}, sig_type: {:?}, support_revocation: {:?}",
        issuer_did,
        schema_json,
        tag,
        sig_type,
        support_revocation
    );
    if settings::indy_mocks_enabled() {
        return Ok((CRED_DEF_ID.to_string(), CRED_DEF_JSON.to_string()));
    }

    let config_json = json!({"support_revocation": support_revocation.unwrap_or(false)}).to_string();

    creddef_libindy::libindy_create_and_store_credential_def(wallet_handle, issuer_did, schema_json, tag, sig_type, &config_json).await
}


#[cfg(test)]
#[cfg(feature = "pool_tests")]
pub mod integration_tests {
    use crate::indy::test_utils::{
        create_and_store_credential, create_and_store_credential_def, create_and_store_nonrevocable_credential_def,
        create_and_write_test_schema, create_indy_proof, create_proof_with_predicate,
    };
    use crate::indy::credential_def::generate_cred_def;
    use crate::indy::ledger::transactions::get_schema_json;
    use crate::indy::primitives::credential_definition::publish_cred_def;
    use crate::indy::primitives::revocation_registry::{generate_rev_reg, publish_rev_reg_def, publish_rev_reg_delta};
    use crate::utils::constants::{DEFAULT_SCHEMA_ATTRS, TAILS_DIR};
    use crate::utils::devsetup::{SetupLibraryWallet, SetupWalletPool};
    use crate::utils::get_temp_dir_path;

    use super::*;

    extern crate serde_json;

    #[tokio::test]
    async fn test_create_cred_def_real() {
        let setup = SetupWalletPool::init().await;

        let (schema_id, _) =
            create_and_write_test_schema(setup.wallet_handle, setup.pool_handle, &setup.institution_did, DEFAULT_SCHEMA_ATTRS).await;
        let (_, schema_json) = get_schema_json(setup.wallet_handle, setup.pool_handle, &schema_id).await.unwrap();

        let (_, cred_def_json) = generate_cred_def(setup.wallet_handle, &setup.institution_did, &schema_json, "tag_1", None, Some(true))
            .await
            .unwrap();
        publish_cred_def(setup.wallet_handle, setup.pool_handle, &setup.institution_did, &cred_def_json)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_rev_reg_def() {
        let setup = SetupWalletPool::init().await;

        let (schema_id, _) =
            create_and_write_test_schema(setup.wallet_handle, setup.pool_handle, &setup.institution_did, DEFAULT_SCHEMA_ATTRS).await;
        let (_, schema_json) = get_schema_json(setup.wallet_handle, setup.pool_handle, &schema_id).await.unwrap();

        let (cred_def_id, cred_def_json) =
            generate_cred_def(setup.wallet_handle, &setup.institution_did, &schema_json, "tag_1", None, Some(true))
                .await
                .unwrap();
        publish_cred_def(setup.wallet_handle, setup.pool_handle, &setup.institution_did, &cred_def_json)
            .await
            .unwrap();
        let (rev_reg_def_id, rev_reg_def_json, rev_reg_entry_json) =
            generate_rev_reg(setup.wallet_handle, &setup.institution_did, &cred_def_id, "tails.txt", 2, "tag1")
                .await
                .unwrap();
        publish_rev_reg_def(setup.wallet_handle, setup.pool_handle, &setup.institution_did, &rev_reg_def_json)
            .await
            .unwrap();
        publish_rev_reg_delta(setup.wallet_handle, setup.pool_handle, &setup.institution_did, &rev_reg_def_id, &rev_reg_entry_json)
            .await
            .unwrap();
    }
}
