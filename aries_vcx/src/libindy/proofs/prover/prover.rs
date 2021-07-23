use crate::error::prelude::*;
use crate::libindy::proofs::proof_request::ProofRequestData;
use crate::libindy::proofs::prover::prover_internal::{build_cred_defs_json_prover, build_requested_credentials_json, build_rev_states_json, build_schemas_json_prover, credential_def_identifiers};
use crate::libindy::utils::anoncreds;
use crate::settings;
use crate::utils::mockdata::mock_settings::get_mock_generate_indy_proof;

pub fn generate_indy_proof(credentials: &str, self_attested_attrs: &str, proof_req_data_json: &str) -> VcxResult<String> {
    trace!("generate_indy_proof >>> credentials: {}, self_attested_attrs: {}", secret!(&credentials), secret!(&self_attested_attrs));

    match get_mock_generate_indy_proof() {
        None => {}
        Some(mocked_indy_proof) => {
            warn!("generate_indy_proof :: returning mocked response");
            return Ok(mocked_indy_proof);
        }
    }

    let proof_request: ProofRequestData = serde_json::from_str(&proof_req_data_json)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize proof request: {}", err)))?;

    let mut credentials_identifiers = credential_def_identifiers(credentials, &proof_request)?;

    let revoc_states_json = build_rev_states_json(&mut credentials_identifiers)?;
    let requested_credentials = build_requested_credentials_json(&credentials_identifiers,
                                                                 self_attested_attrs,
                                                                 &proof_request)?;

    let schemas_json = build_schemas_json_prover(&credentials_identifiers)?;
    let credential_defs_json = build_cred_defs_json_prover(&credentials_identifiers)?;

    let proof = anoncreds::libindy_prover_create_proof(&proof_req_data_json,
                                                       &requested_credentials,
                                                       settings::DEFAULT_LINK_SECRET_ALIAS,
                                                       &schemas_json,
                                                       &credential_defs_json,
                                                       Some(&revoc_states_json))?;
    Ok(proof)
}
