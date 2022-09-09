use vdrtools_sys::WalletHandle;
use serde_json;

use crate::error::{VcxError, VcxErrorKind, VcxResult};
use crate::libindy::utils::wallet::{
    add_wallet_record, delete_wallet_record, get_wallet_record, update_wallet_record_value,
};

static WALLET_RECORD_TYPE: &str = "cache";
static RECORD_ID_PREFIX: &str = "rev_reg_delta:";

///
/// Returns stored revocation registry delta record
///
/// # Arguments
/// `rev_reg_id`: revocation registry id
///
/// # Returns
/// Revocation registry delta json as a string
pub async fn get_rev_reg_delta(wallet_handle: WalletHandle, rev_reg_id: &str) -> Option<String> {
    debug!("Getting get_rev_reg_delta for rev_reg_id {}", rev_reg_id);

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
                    "Unable to convert rev_reg_delta cache for rev_reg_id: {}, json: {}, error: {}",
                    rev_reg_id, json, err
                );
                None
            }
        },
        Err(err) => {
            warn!(
                "Unable to get rev_reg_delta cache for rev_reg_id: {}, error: {}",
                rev_reg_id, err
            );
            None
        }
    }
}

///
///
/// Rewrites or creates revocation registry delta record
///
/// # Arguments
/// `rev_reg_id`: revocation registry id.
/// `cache`: Cache object.
///
pub async fn set_rev_reg_delta(wallet_handle: WalletHandle, rev_reg_id: &str, cache: &str) -> VcxResult<()> {
    debug!(
        "Setting set_rev_reg_delta for rev_reg_id {}, cache {}",
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
        Err(_) => Err(VcxError::from(VcxErrorKind::SerializationError)),
    }
}

///
///
/// Clears the stored reovcation registry delta record
/// Errors are silently ignored.
///
/// # Arguments
/// `rev_reg_id`: revocation registry id.
/// `cache`: Cache object.
///
pub async fn clear_rev_reg_delta(wallet_handle: WalletHandle, rev_reg_id: &str) -> VcxResult<String> {
    debug!("Clearing clear_rev_reg_delta for rev_reg_id {}", rev_reg_id);
    if let Some(last_delta) = get_rev_reg_delta(wallet_handle, rev_reg_id).await {
        debug!("Got last delta = {}", last_delta);
        let wallet_id = format!("{}{}", RECORD_ID_PREFIX, rev_reg_id);
        delete_wallet_record(wallet_handle, WALLET_RECORD_TYPE, &wallet_id).await?;
        debug!("Record with id {} deleted", wallet_id);
        Ok(last_delta)
    } else {
        Err(VcxError::from(VcxErrorKind::IOError))
    }
}
