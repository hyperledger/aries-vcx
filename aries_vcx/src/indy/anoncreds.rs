use vdrtools::{Locator, SearchHandle};

use crate::errors::error::prelude::*;

pub(super) async fn blob_storage_open_reader(base_dir: &str) -> VcxResult<i32> {
    let tails_config = json!(
        {
            "base_dir":    base_dir,
            "uri_pattern": ""         // TODO remove, unused
        }
    )
    .to_string();

    let res = Locator::instance()
        .blob_storage_controller
        .open_reader("default".into(), tails_config)
        .await?;

    Ok(res)
}

pub(super) async fn close_search_handle(search_handle: SearchHandle) -> VcxResult<()> {
    Locator::instance()
        .prover_controller
        .close_credentials_search_for_proof_req(search_handle)
        .await?;

    Ok(())
}

pub async fn generate_nonce() -> VcxResult<String> {
    let res = Locator::instance().verifier_controller.generate_nonce()?;

    Ok(res)
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use vdrtools::WalletHandle;

    use crate::{
        indy::ledger::transactions::get_schema_json,
        utils::{
            constants::{SCHEMA_ID, SCHEMA_JSON},
            devsetup::SetupMocks,
        },
    };

    #[tokio::test]
    async fn from_ledger_schema_id() {
        let _setup = SetupMocks::init();
        let (id, retrieved_schema) = get_schema_json(WalletHandle(0), 1, SCHEMA_ID).await.unwrap();
        assert_eq!(&retrieved_schema, SCHEMA_JSON);
        assert_eq!(&id, SCHEMA_ID);
    }
}

#[cfg(feature = "pool_tests")]
#[cfg(test)]
pub mod integration_tests {

    use crate::{
        common::test_utils::{create_and_store_credential, indy_handles_to_profile},
        indy::primitives::revocation_registry::libindy_issuer_revoke_credential,
        utils::{constants::TAILS_DIR, devsetup::SetupWalletPool, get_temp_dir_path},
    };

    #[tokio::test]
    async fn test_issuer_revoke_credential() {
        SetupWalletPool::run(|setup| async move {
            let rc = libindy_issuer_revoke_credential(
                setup.wallet_handle,
                get_temp_dir_path(TAILS_DIR).to_str().unwrap(),
                "",
                "",
            )
            .await;
            assert!(rc.is_err());

            let profile = indy_handles_to_profile(setup.wallet_handle, setup.pool_handle);
            let (_, _, _, _, _, _, _, _, rev_reg_id, cred_rev_id, _) = create_and_store_credential(
                &profile,
                &profile,
                &setup.institution_did,
                crate::utils::constants::DEFAULT_SCHEMA_ATTRS,
            )
            .await;

            let rc = libindy_issuer_revoke_credential(
                setup.wallet_handle,
                get_temp_dir_path(TAILS_DIR).to_str().unwrap(),
                &rev_reg_id,
                &cred_rev_id,
            )
            .await;

            assert!(rc.is_ok());
        })
        .await;
    }
}
