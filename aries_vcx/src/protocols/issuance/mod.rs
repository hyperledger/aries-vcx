use indy_sys::WalletHandle;

use crate::error::prelude::*;
use crate::global::settings;
use crate::libindy::utils::anoncreds::get_cred_def_json;
use crate::protocols::issuance::actions::CredentialIssuanceAction;

pub mod actions;
pub mod holder;
pub mod issuer;

pub fn verify_thread_id(thread_id: &str, message: &CredentialIssuanceAction) -> VcxResult<()> {
    if !settings::indy_mocks_enabled() && !message.thread_id_matches(thread_id) {
        return Err(VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!(
                "Cannot handle message {:?}: thread id does not match, expected {:?}",
                message, thread_id
            ),
        ));
    };
    Ok(())
}

pub async fn is_cred_def_revokable(wallet_handle: WalletHandle, cred_def_id: &str) -> VcxResult<bool> {
    let pool_handle = crate::global::pool::get_main_pool_handle()?;
    let (_, cred_def_json) = get_cred_def_json(wallet_handle, pool_handle, cred_def_id).await.map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::InvalidLedgerResponse,
            format!("Failed to obtain credential definition from ledger or cache: {}", err),
        )
    })?;
    let parsed_cred_def: serde_json::Value = serde_json::from_str(&cred_def_json).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!(
                "Failed deserialize credential definition json {}\nError: {}",
                cred_def_json, err
            ),
        )
    })?;
    Ok(!parsed_cred_def["value"]["revocation"].is_null())
}
