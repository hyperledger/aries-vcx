pub mod prover;

use vdrtools::Locator;

use crate::errors::error::VcxCoreResult;
use crate::global::settings;
use crate::indy;
use crate::utils::constants::REV_STATE_JSON;
use crate::indy::utils::parse_and_validate;

pub async fn libindy_prover_create_revocation_state(
    tails_file_path: &str,
    rev_reg_def_json: &str,
    rev_reg_delta_json: &str,
    timestamp: u64,
    cred_rev_id: &str,
) -> VcxCoreResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(REV_STATE_JSON.to_string());
    }

    let blob_handle = indy::anoncreds::blob_storage_open_reader(tails_file_path).await?;

    let res = Locator::instance()
        .prover_controller
        .create_revocation_state(
            blob_handle,
            parse_and_validate(rev_reg_def_json)?,
            parse_and_validate(rev_reg_delta_json)?,
            timestamp,
            cred_rev_id.into(),
        )
        .await?;

    Ok(res)
}
