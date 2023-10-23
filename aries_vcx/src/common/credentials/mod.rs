use std::collections::HashMap;

use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds, ledger::base_ledger::AnoncredsLedgerRead,
    wallet::base_wallet::BaseWallet,
};
use time::OffsetDateTime;

use super::primitives::revocation_registry_delta::RevocationRegistryDelta;
use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

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

pub async fn get_cred_rev_id(
    wallet: &impl BaseWallet,
    anoncreds: &impl BaseAnonCreds,
    cred_id: &str,
) -> VcxResult<String> {
    let cred_json = anoncreds.prover_get_credential(wallet, cred_id).await?;
    let prover_cred = serde_json::from_str::<ProverCredential>(&cred_json).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Failed to deserialize anoncreds credential: {}", err),
        )
    })?;
    prover_cred.cred_rev_id.ok_or(AriesVcxError::from_msg(
        AriesVcxErrorKind::InvalidRevocationDetails,
        "Credenial revocation id missing on credential - is this credential revokable?",
    ))
}

pub async fn is_cred_revoked(
    ledger: &impl AnoncredsLedgerRead,
    rev_reg_id: &str,
    rev_id: &str,
) -> VcxResult<bool> {
    let to = Some(OffsetDateTime::now_utc().unix_timestamp() as u64 + 100);
    let (_, rev_reg_delta_json, _) = ledger.get_rev_reg_delta_json(rev_reg_id, None, to).await?;
    let rev_reg_delta = RevocationRegistryDelta::create_from_ledger(&rev_reg_delta_json).await?;
    Ok(rev_reg_delta
        .revoked()
        .iter()
        .any(|s| s.to_string().eq(rev_id)))
}