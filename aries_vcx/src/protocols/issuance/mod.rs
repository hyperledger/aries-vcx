use crate::error::prelude::*;
use crate::libindy::utils::anoncreds::get_cred_def_json;
use crate::protocols::issuance::actions::CredentialIssuanceAction;
use crate::settings;

pub mod issuer;
pub mod holder;
pub mod actions;

pub fn verify_thread_id(thread_id: &str, message: &CredentialIssuanceAction) -> VcxResult<()> {
    if !settings::indy_mocks_enabled() && !message.thread_id_matches(thread_id) {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot handle message {:?}: thread id does not match, expected {:?}", message, thread_id)));
    };
    Ok(())
}

pub fn is_cred_def_revokable(cred_def_id: &str) -> VcxResult<bool> {
    let (_, cred_def_json) = get_cred_def_json(cred_def_id)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("Failed to obtain credential definition from ledger or cache: {}", err)))?;
    let parsed_cred_def: serde_json::Value = serde_json::from_str(&cred_def_json)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed deserialize credential definition json {}\nError: {}", cred_def_json, err)))?;
    Ok(!parsed_cred_def["value"]["revocation"].is_null())
}
