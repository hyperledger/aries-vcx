use vdrtools_sys::WalletHandle;
use vdrtools::anoncreds;
use serde_json::{Map, Value};
use crate::error::{VcxError, VcxErrorKind, VcxResult};
use crate::global::settings;
use crate::{indy, utils};
use crate::utils::constants::{PROOF_REQUESTED_PREDICATES, REQUESTED_ATTRIBUTES, REV_STATE_JSON};
use crate::utils::mockdata::mock_settings::get_mock_creds_retrieved_for_proof_request;

pub mod prover;
mod prover_internal;

pub async fn libindy_prover_create_revocation_state(
    rev_reg_def_json: &str,
    rev_reg_delta_json: &str,
    cred_rev_id: &str,
    tails_file: &str,
) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(REV_STATE_JSON.to_string());
    }

    let blob_handle = indy::anoncreds::blob_storage_open_reader(tails_file).await?;

    anoncreds::create_revocation_state(blob_handle, rev_reg_def_json, rev_reg_delta_json, 100, cred_rev_id)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_prover_update_revocation_state(
    rev_reg_def_json: &str,
    rev_state_json: &str,
    rev_reg_delta_json: &str,
    cred_rev_id: &str,
    tails_file: &str,
) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(REV_STATE_JSON.to_string());
    }

    let blob_handle = indy::anoncreds::blob_storage_open_reader(tails_file).await?;

    anoncreds::update_revocation_state(
        blob_handle,
        rev_state_json,
        rev_reg_def_json,
        rev_reg_delta_json,
        100,
        cred_rev_id,
    )
        .await
        .map_err(VcxError::from)
}
