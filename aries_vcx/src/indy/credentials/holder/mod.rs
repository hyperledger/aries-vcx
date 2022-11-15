use vdrtools_sys::WalletHandle;
use vdrtools::anoncreds;
use crate::error::{VcxError, VcxResult};
use crate::global::settings;
use crate::utils;

pub async fn libindy_prover_store_credential(
    wallet_handle: WalletHandle,
    cred_id: Option<&str>,
    cred_req_meta: &str,
    cred_json: &str,
    cred_def_json: &str,
    rev_reg_def_json: Option<&str>,
) -> VcxResult<String> {
    trace!("libindy_prover_store_credential >>> \
            cred_id: {:?}, \
            cred_req_meta: {}, \
            cred_json: {}, \
            cred_def_json: {}, \
            rev_reg_def_json: {:?}",
           cred_id, cred_req_meta, cred_json, cred_def_json, rev_reg_def_json,
    );

    if settings::indy_mocks_enabled() {
        return Ok("cred_id".to_string());
    }

    anoncreds::prover_store_credential(
        wallet_handle,
        cred_id,
        cred_req_meta,
        cred_json,
        cred_def_json,
        rev_reg_def_json,
    )
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_prover_get_credential(
    wallet_handle: WalletHandle,
    cred_id: &str,
) -> VcxResult<String> {
    trace!("libindy_prover_get_credential >>> \
            cred_id: {:?}",
           cred_id,
    );

    anoncreds::prover_get_credential(
        wallet_handle,
        cred_id,
    )
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_prover_delete_credential(wallet_handle: WalletHandle, cred_id: &str) -> VcxResult<()> {
    anoncreds::prover_delete_credential(wallet_handle, cred_id)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_prover_create_master_secret(
    wallet_handle: WalletHandle,
    master_secret_id: &str,
) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(settings::DEFAULT_LINK_SECRET_ALIAS.to_string());
    }

    anoncreds::prover_create_master_secret(wallet_handle, Some(master_secret_id))
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_prover_create_credential_req(
    wallet_handle: WalletHandle,
    prover_did: &str,
    credential_offer_json: &str,
    credential_def_json: &str,
    master_secret_name: &str
) -> VcxResult<(String, String)> {
    if settings::indy_mocks_enabled() {
        return Ok((utils::constants::CREDENTIAL_REQ_STRING.to_owned(), String::new()));
    }

    anoncreds::prover_create_credential_req(
        wallet_handle,
        prover_did,
        credential_offer_json,
        credential_def_json,
        master_secret_name,
    )
        .await
        .map_err(VcxError::from)
}
