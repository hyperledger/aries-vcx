use serde_json;
use vdrtools::WalletHandle;

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    indy::wallet::{add_wallet_record, delete_wallet_record, get_wallet_record, update_wallet_record_value},
};

static WALLET_RECORD_TYPE: &str = "cache";
static RECORD_ID_PREFIX: &str = "rev_reg_delta:";

/// Returns stored revocation registry delta record
///
/// # Arguments
/// `rev_reg_id`: revocation registry id
///
/// # Returns
/// Revocation registry delta json as a string
/// todo: return VcxResult<Option<String>>, don't swallow errors
pub async fn get_rev_reg_delta(wallet_handle: WalletHandle, rev_reg_id: &str) -> Option<String> {
    debug!(
        "get_rev_reg_delta >> Getting revocation registry delta for rev_reg_id {}",
        rev_reg_id
    );

    let wallet_id = format!("{}{}", RECORD_ID_PREFIX, rev_reg_id);

    match get_wallet_record(
        wallet_handle,
        WALLET_RECORD_TYPE,
        &wallet_id,
        &json!({"retrieveType": false, "retrieveValue": true, "retrieveTags": false}).to_string(),
    )
    .await
    {
        Ok(json) => match serde_json::from_str(&json).and_then(|x: serde_json::Value| {
            serde_json::from_str(
                x.get("value")
                    .unwrap_or(&serde_json::Value::Null)
                    .as_str()
                    .unwrap_or(""),
            )
        }) {
            Ok(cache) => cache,
            Err(err) => {
                warn!(
                    "get_rev_reg_delta >> Unable to convert rev_reg_delta cache for rev_reg_id: {}, json: {}, error: \
                     {}",
                    rev_reg_id, json, err
                );
                None
            }
        },
        Err(err) => {
            warn!(
                "get_rev_reg_delta >> Unable to get rev_reg_delta cache for rev_reg_id: {}, error: {}",
                rev_reg_id, err
            );
            None
        }
    }
}

/// Rewrites or creates revocation registry delta record
///
/// # Arguments
/// `rev_reg_id`: revocation registry id.
/// `cache`: Cache object.
pub async fn set_rev_reg_delta(wallet_handle: WalletHandle, rev_reg_id: &str, cache: &str) -> VcxResult<()> {
    debug!(
        "set_rev_reg_delta >> Setting store revocation registry delta for revocation registry {} to new value: {}",
        rev_reg_id, cache
    );
    match serde_json::to_string(cache) {
        Ok(json) => {
            let wallet_id = format!("{}{}", RECORD_ID_PREFIX, rev_reg_id);
            match update_wallet_record_value(wallet_handle, WALLET_RECORD_TYPE, &wallet_id, &json)
                .await
                .or(add_wallet_record(wallet_handle, WALLET_RECORD_TYPE, &wallet_id, &json, None).await)
            {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        }
        Err(_) => Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Expected cache argument to be valid json. Found instead: {}", cache),
        )),
    }
}

/// Clears the stored revocation registry delta record
///
/// # Arguments
/// `rev_reg_id`: revocation registry id.
/// `cache`: Cache object.
pub async fn clear_rev_reg_delta(wallet_handle: WalletHandle, rev_reg_id: &str) -> VcxResult<String> {
    debug!(
        "clear_rev_reg_delta >> Clear revocation registry delta for rev_reg_id {}",
        rev_reg_id
    );
    if let Some(last_delta) = get_rev_reg_delta(wallet_handle, rev_reg_id).await {
        let wallet_id = format!("{}{}", RECORD_ID_PREFIX, rev_reg_id);
        delete_wallet_record(wallet_handle, WALLET_RECORD_TYPE, &wallet_id).await?;
        info!(
            "clear_rev_reg_delta >> Cleared stored revocation delta for revocation registry {}, wallet record: ${}",
            rev_reg_id, wallet_id
        );
        Ok(last_delta)
    } else {
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::IOError,
            format!(
                "Couldn't fetch delta for rev_reg_id {} before deletion, deletion skipped",
                rev_reg_id
            ),
        ))
    }
}
