use vdrtools::{Locator};

use crate::error::VcxResult;

pub async fn libindy_verifier_verify_proof(
    proof_req_json: &str,
    proof_json: &str,
    schemas_json: &str,
    credential_defs_json: &str,
    rev_reg_defs_json: &str,
    rev_regs_json: &str,
) -> VcxResult<bool> {

    let res = Locator::instance()
        .verifier_controller
        .verify_proof(
            serde_json::from_str(proof_req_json)?,
            serde_json::from_str(proof_json)?,
            serde_json::from_str(schemas_json)?,
            serde_json::from_str(credential_defs_json)?,
            serde_json::from_str(rev_reg_defs_json)?,
            serde_json::from_str(rev_regs_json)?,
        )?;

    Ok(res)
}
