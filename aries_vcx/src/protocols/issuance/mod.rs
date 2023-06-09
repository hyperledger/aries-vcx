use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use std::sync::Arc;

use crate::errors::error::prelude::*;
use crate::global::settings;
use crate::protocols::issuance::actions::CredentialIssuanceAction;

pub mod actions;
pub mod holder;
pub mod issuer;

pub fn verify_thread_id(thread_id: &str, message: &CredentialIssuanceAction) -> VcxResult<()> {
    if !settings::indy_mocks_enabled() && !message.thread_id_matches(thread_id) {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!(
                "Cannot handle message {:?}: thread id does not match, expected {:?}",
                message, thread_id
            ),
        ));
    };
    Ok(())
}

pub async fn is_cred_def_revokable(ledger: &Arc<dyn AnoncredsLedgerRead>, cred_def_id: &str) -> VcxResult<bool> {
    let cred_def_json = ledger.get_cred_def(cred_def_id, None).await.map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidLedgerResponse,
            format!("Failed to obtain credential definition from ledger or cache: {}", err),
        )
    })?;
    let parsed_cred_def: serde_json::Value = serde_json::from_str(&cred_def_json).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!(
                "Failed deserialize credential definition json {}\nError: {}",
                cred_def_json, err
            ),
        )
    })?;
    Ok(!parsed_cred_def["value"]["revocation"].is_null())
}
