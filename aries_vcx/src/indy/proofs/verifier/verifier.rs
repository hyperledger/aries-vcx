use vdrtools::Locator;

use crate::{errors::error::VcxResult, utils::parse_and_validate};

pub async fn libindy_verifier_verify_proof(
    proof_req_json: &str,
    proof_json: &str,
    schemas_json: &str,
    credential_defs_json: &str,
    rev_reg_defs_json: &str,
    rev_regs_json: &str,
) -> VcxResult<bool> {
    let res = Locator::instance().verifier_controller.verify_proof(
        parse_and_validate(proof_req_json)?,
        parse_and_validate(proof_json)?,
        serde_json::from_str(schemas_json)?,
        serde_json::from_str(credential_defs_json)?,
        serde_json::from_str(rev_reg_defs_json)?,
        serde_json::from_str(rev_regs_json)?,
    )?;

    Ok(res)
}
