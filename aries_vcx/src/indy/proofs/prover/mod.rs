use vdrtools::Locator;

use crate::error::VcxResult;
use crate::global::settings;
use crate::indy;
use crate::utils::parse_and_validate;
use crate::utils::constants::REV_STATE_JSON;

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

    let blob_handle =
        indy::anoncreds::blob_storage_open_reader(tails_file)
        .await?;

    let res = Locator::instance()
        .prover_controller
        .create_revocation_state(
            blob_handle,
            parse_and_validate(rev_reg_def_json)?,
            parse_and_validate(rev_reg_delta_json)?,
            100,
            cred_rev_id.into(),
        ).await?;

    Ok(res)
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

    let res = Locator::instance()
        .prover_controller
        .update_revocation_state(
            blob_handle,
            parse_and_validate(rev_state_json)?,
            parse_and_validate(rev_reg_def_json)?,
            parse_and_validate(rev_reg_delta_json)?,
            100,
            cred_rev_id.into(),
        ).await?;

    Ok(res)
}
