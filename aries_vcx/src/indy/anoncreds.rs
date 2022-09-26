use vdrtools::future::TryFutureExt;
use vdrtools::{anoncreds, blob_storage, ledger};
use vdrtools_sys::{PoolHandle, WalletHandle};
use serde_json;
use serde_json::{map::Map, Value};
use time;

use crate::error::prelude::*;
use crate::global::settings;
use crate::indy::wallet_non_secrets::{clear_rev_reg_delta, get_rev_reg_delta, set_rev_reg_delta};
use crate::indy::ledger::transactions::*;
use crate::indy::ledger::transactions::sign_and_submit_to_ledger;
use crate::indy::primitives::{credential_schema, revocation_registry};
use crate::indy::primitives::revocation_registry::RevocationRegistryDefinition;
use crate::indy::utils::LibindyMock;
use crate::utils;
use crate::utils::constants::{
    CRED_DEF_ID, CRED_DEF_JSON, CRED_DEF_REQ, rev_def_json, REV_REG_DELTA_JSON, REV_REG_ID, REV_REG_JSON,
    REVOC_REG_TYPE, SCHEMA_ID, SCHEMA_JSON, SCHEMA_TXN,
};
use crate::utils::constants::{
    ATTRS, LIBINDY_CRED_OFFER, PROOF_REQUESTED_PREDICATES, REQUESTED_ATTRIBUTES, REV_STATE_JSON,
};
use crate::utils::mockdata::mock_settings::get_mock_creds_retrieved_for_proof_request;
use crate::utils::random::generate_random_did;

pub(super) async fn blob_storage_open_reader(base_dir: &str) -> VcxResult<i32> {
    let tails_config = json!({"base_dir": base_dir,"uri_pattern": ""}).to_string();
    blob_storage::open_reader("default", &tails_config)
        .await
        .map_err(VcxError::from)
}

pub(super) async fn close_search_handle(search_handle: i32) -> VcxResult<()> {
    anoncreds::prover_close_credentials_search_for_proof_req(search_handle)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_to_unqualified(entity: &str) -> VcxResult<String> {
    anoncreds::to_unqualified(entity).await.map_err(VcxError::from)
}

pub async fn generate_nonce() -> VcxResult<String> {
    anoncreds::generate_nonce().await.map_err(VcxError::from)
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use vdrtools_sys::{PoolHandle, WalletHandle};

    use crate::indy::ledger::transactions::get_schema_json;
    use crate::utils::constants::{SCHEMA_ID, SCHEMA_JSON};
    use crate::utils::devsetup::SetupMocks;

    #[tokio::test]
    async fn from_ledger_schema_id() {
        let _setup = SetupMocks::init();
        let (id, retrieved_schema) = get_schema_json(WalletHandle(0), 1, SCHEMA_ID).await.unwrap();
        assert_eq!(&retrieved_schema, SCHEMA_JSON);
        assert_eq!(&id, SCHEMA_ID);
    }
}

#[cfg(test)]
#[cfg(feature = "pool_tests")]
pub mod integration_tests {
    use crate::indy::test_utils::{
        create_and_store_credential, create_and_store_credential_def, create_and_store_nonrevocable_credential_def,
        create_and_write_test_schema, create_indy_proof, create_proof_with_predicate,
    };
    use crate::indy::primitives::credential_definition::generate_cred_def;
    use crate::indy::proofs::prover::prover::libindy_prover_get_credentials_for_proof_req;
    use crate::indy::primitives::revocation_registry::{generate_rev_reg, libindy_issuer_revoke_credential, publish_local_revocations, revoke_credential_local};
    use crate::utils::constants::TAILS_DIR;
    use crate::utils::devsetup::{SetupLibraryWallet, SetupWalletPool};
    use crate::utils::get_temp_dir_path;

    use super::*;

    extern crate serde_json;


    #[tokio::test]
    async fn tests_libindy_returns_error_if_proof_request_is_malformed() {
        let setup = SetupLibraryWallet::init().await;

        let proof_req = "{";
        let result = libindy_prover_get_credentials_for_proof_req(setup.wallet_handle, &proof_req).await;
        assert_eq!(result.unwrap_err().kind(), VcxErrorKind::InvalidProofRequest);
    }

    #[tokio::test]
    async fn tests_libindy_prover_get_credentials() {
        let setup = SetupLibraryWallet::init().await;

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
        let _result = libindy_prover_get_credentials_for_proof_req(setup.wallet_handle, &proof_req)
            .await
            .unwrap();

        let result_malformed_json = libindy_prover_get_credentials_for_proof_req(setup.wallet_handle, "{}")
            .await
            .unwrap_err();
        assert_eq!(result_malformed_json.kind(), VcxErrorKind::InvalidAttributesStructure);
    }

    #[tokio::test]
    async fn test_issuer_revoke_credential() {
        let setup = SetupWalletPool::init().await;

        let rc = libindy_issuer_revoke_credential(
            setup.wallet_handle,
            get_temp_dir_path(TAILS_DIR).to_str().unwrap(),
            "",
            "",
        )
            .await;
        assert!(rc.is_err());

        let (_, _, _, _, _, _, _, _, rev_reg_id, cred_rev_id) =
            create_and_store_credential(setup.wallet_handle, setup.pool_handle, &setup.institution_did, utils::constants::DEFAULT_SCHEMA_ATTRS).await;
        let rc = libindy_issuer_revoke_credential(
            setup.wallet_handle,
            get_temp_dir_path(TAILS_DIR).to_str().unwrap(),
            &rev_reg_id,
            &cred_rev_id,
        )
            .await;

        assert!(rc.is_ok());
    }

    #[tokio::test]
    async fn test_revoke_credential() {
        let setup = SetupWalletPool::init().await;

        let (_, _, _, _, _, _, _, _, rev_reg_id, cred_rev_id) =
            create_and_store_credential(setup.wallet_handle, setup.pool_handle, &setup.institution_did, utils::constants::DEFAULT_SCHEMA_ATTRS).await;

        let (_, first_rev_reg_delta, first_timestamp) = get_rev_reg_delta_json(setup.pool_handle, &rev_reg_id, None, None).await.unwrap();
        let (_, test_same_delta, test_same_timestamp) = get_rev_reg_delta_json(setup.pool_handle, &rev_reg_id, None, None).await.unwrap();

        assert_eq!(first_rev_reg_delta, test_same_delta);
        assert_eq!(first_timestamp, test_same_timestamp);

        revoke_credential_local(
            setup.wallet_handle,
            get_temp_dir_path(TAILS_DIR).to_str().unwrap(),
            &rev_reg_id,
            &cred_rev_id
        )
            .await
            .unwrap();

        publish_local_revocations(setup.wallet_handle, setup.pool_handle, &setup.institution_did, &rev_reg_id)
            .await
            .unwrap();

        // Delta should change after revocation
        let (_, second_rev_reg_delta, _) = get_rev_reg_delta_json(setup.pool_handle, &rev_reg_id, Some(first_timestamp + 1), None)
            .await
            .unwrap();

        assert_ne!(first_rev_reg_delta, second_rev_reg_delta);
    }
}
