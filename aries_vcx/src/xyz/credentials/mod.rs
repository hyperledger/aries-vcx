use std::{collections::HashMap, sync::Arc};

use time::get_time;

use crate::{core::profile::profile::Profile, error::{VcxResult, VcxError, VcxErrorKind}};

use super::primitives::revocation_registry_delta::RevocationRegistryDelta;

pub mod encoding;

#[derive(Serialize, Deserialize)]
struct ProverCredential {
    referent: String,
    attrs: HashMap<String, String>,
    schema_id: String,
    cred_def_id: String,
    rev_reg_id: Option<String>,
    cred_rev_id: Option<String>,
}

pub async fn get_cred_rev_id(profile: &Arc<dyn Profile>, cred_id: &str) -> VcxResult<String> {
    let anoncreds = Arc::clone(profile).inject_anoncreds();
    let cred_json = anoncreds.prover_get_credential(cred_id).await?;
    let prover_cred = serde_json::from_str::<ProverCredential>(&cred_json).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!("Failed to deserialize anoncreds credential: {}", err),
        )
    })?;
    prover_cred.cred_rev_id.ok_or(VcxError::from_msg(
        VcxErrorKind::InvalidRevocationDetails,
        "Credenial revocation id missing on credential - is this credential revokable?",
    ))
}

pub async fn is_cred_revoked(
    profile: &Arc<dyn Profile>,
    rev_reg_id: &str,
    rev_id: &str,
) -> VcxResult<bool> {
    let from = None;
    let to = Some(get_time().sec as u64 + 100);
    let rev_reg_delta = RevocationRegistryDelta::create_from_ledger(profile, rev_reg_id, from, to).await?;
    Ok(rev_reg_delta.revoked().iter().any(|s| s.to_string().eq(rev_id)))
}
