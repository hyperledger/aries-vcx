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
pub struct ProverCredential {
    pub referent: String,
    pub attrs: HashMap<String, String>,
    pub schema_id: String,
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub cred_rev_id: Option<u32>,
}

pub async fn get_cred_rev_id(
    wallet: &impl BaseWallet,
    anoncreds: &impl BaseAnonCreds,
    cred_id: &str,
) -> VcxResult<u32> {
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
    rev_id: u32,
) -> VcxResult<bool> {
    let to = Some(OffsetDateTime::now_utc().unix_timestamp() as u64 + 100);
    let (rev_reg_delta_json, _) = ledger
        .get_rev_reg_delta_json(&rev_reg_id.try_into()?, None, to)
        .await?;
    let rev_reg_delta =
        RevocationRegistryDelta::create_from_ledger(&serde_json::to_string(&rev_reg_delta_json)?)
            .await?;
    Ok(rev_reg_delta.revoked().iter().any(|s| s.eq(&rev_id)))
}
