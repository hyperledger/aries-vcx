pub mod prover;

use vdrtools::anoncreds;
use crate::error::{VcxError, VcxResult};
use crate::global::settings;
use crate::indy;
use crate::utils::constants::REV_STATE_JSON;

pub async fn libindy_prover_create_revocation_state(
    tails_file_path: &str,
    rev_reg_def_json: &str,
    rev_reg_delta_json: &str,
    timestamp: u64,
    cred_rev_id: &str,
) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(REV_STATE_JSON.to_string());
    }

    let blob_handle = indy::anoncreds::blob_storage_open_reader(tails_file_path).await?;

    anoncreds::create_revocation_state(blob_handle, rev_reg_def_json, rev_reg_delta_json, timestamp, cred_rev_id)
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
