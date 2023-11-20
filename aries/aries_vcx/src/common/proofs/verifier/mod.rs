mod verifier_internal;

use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds, ledger::base_ledger::AnoncredsLedgerRead,
};

use crate::{
    common::proofs::verifier::verifier_internal::{
        build_cred_defs_json_verifier, build_rev_reg_defs_json, build_rev_reg_json,
        build_schemas_json_verifier, get_credential_info, validate_proof_revealed_attributes,
    },
    errors::error::prelude::*,
};

pub async fn validate_indy_proof(
    ledger: &impl AnoncredsLedgerRead,
    anoncreds: &impl BaseAnonCreds,
    proof_json: &str,
    proof_req_json: &str,
) -> VcxResult<bool> {
    validate_proof_revealed_attributes(proof_json)?;

    let credential_data = get_credential_info(proof_json)?;
    debug!("validate_indy_proof >> credential_data: {credential_data:?}");
    let credential_defs_json = build_cred_defs_json_verifier(ledger, &credential_data).await?;
    let schemas_json = build_schemas_json_verifier(ledger, &credential_data).await?;
    let rev_reg_defs_json = build_rev_reg_defs_json(ledger, &credential_data)
        .await
        .unwrap_or(json!({}).to_string());
    let rev_regs_json = build_rev_reg_json(ledger, &credential_data)
        .await
        .unwrap_or(json!({}).to_string());

    debug!("validate_indy_proof >> credential_defs_json: {credential_defs_json}");
    debug!("validate_indy_proof >> schemas_json: {schemas_json}");
    trace!("validate_indy_proof >> proof_json: {proof_json}");
    debug!("validate_indy_proof >> proof_req_json: {proof_req_json}");
    debug!("validate_indy_proof >> rev_reg_defs_json: {rev_reg_defs_json}");
    debug!("validate_indy_proof >> rev_regs_json: {rev_regs_json}");
    anoncreds
        .verifier_verify_proof(
            proof_req_json,
            proof_json,
            &schemas_json,
            &credential_defs_json,
            &rev_reg_defs_json,
            &rev_regs_json,
        )
        .await
        .map_err(|err| err.into())
}
