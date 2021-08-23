use serde_json;

use crate::error::{VcxError, VcxErrorKind, VcxResult};
use crate::libindy::utils::wallet::{add_record, delete_record, get_record, update_record_value};

static CACHE_TYPE: &str = "cache";
static REV_REG_DELTA_CACHE_PREFIX: &str = "rev_reg_delta:";
static REV_REG_IDS_CACHE_PREFIX: &str = "rev_reg_ids:";

// TODO: Maybe we need to persist more info
#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
pub struct RevRegIdsCache {
    pub rev_reg_ids: Vec<String>
}

fn set_rev_reg_ids_cache(cred_def_id: &str, cache: &str) -> VcxResult<()> {
    debug!("Setting rev_reg_ids for cred_def_id {}, cache {}", cred_def_id, cache);
    match serde_json::to_string(cache) {
        Ok(json) => {
            let wallet_id = format!("{}{}", REV_REG_IDS_CACHE_PREFIX, cred_def_id);
            match update_record_value(CACHE_TYPE, &wallet_id, &json)
                .or(add_record(CACHE_TYPE, &wallet_id, &json, None)) {
                Ok(_) => Ok(()),
                Err(err) => Err(err)
            }
        }
        Err(_) => {
            Err(VcxError::from(VcxErrorKind::SerializationError))
        }
    }
}

fn get_rev_reg_ids_cache(cred_def_id: &str) -> Option<RevRegIdsCache> {
    debug!("Getting rev_reg_delta_cache for cred_def_id {}", cred_def_id);
    let wallet_id = format!("{}{}", REV_REG_IDS_CACHE_PREFIX, cred_def_id);

    match get_record(CACHE_TYPE, &wallet_id, &json!({"retrieveType": false, "retrieveValue": true, "retrieveTags": false}).to_string()) {
        Ok(json) => {
            match serde_json::from_str(&json)
                .and_then(|x: serde_json::Value|
                    serde_json::from_str(x.get("value").unwrap_or(&serde_json::Value::Null).as_str().unwrap_or(""))) {
                Ok(cache) => cache,
                Err(err) => {
                    warn!("Unable to convert rev_reg_ids cache for cred_def_id: {}, json: {}, error: {}", cred_def_id, json, err);
                    None
                }
            }
        }
        Err(err) => {
            warn!("Unable to get rev_reg_ids cache for cred_def_id: {}, error: {}", cred_def_id, err);
            None
        }
    }
}

///
/// Returns the rev reg delta cache.
///
/// # Arguments
/// `rev_reg_id`: revocation registry id
///
/// # Returns
/// Revocation registry delta json as a string
pub fn get_rev_reg_delta_cache(rev_reg_id: &str) -> Option<String> {
    debug!("Getting rev_reg_delta_cache for rev_reg_id {}", rev_reg_id);

    let wallet_id = format!("{}{}", REV_REG_DELTA_CACHE_PREFIX, rev_reg_id);

    match get_record(CACHE_TYPE, &wallet_id, &json!({"retrieveType": false, "retrieveValue": true, "retrieveTags": false}).to_string()) {
        Ok(json) => {
            match serde_json::from_str(&json)
                .and_then(|x: serde_json::Value|
                    serde_json::from_str(x.get("value").unwrap_or(&serde_json::Value::Null).as_str().unwrap_or(""))) {
                Ok(cache) => cache,
                Err(err) => {
                    warn!("Unable to convert rev_reg_delta cache for rev_reg_id: {}, json: {}, error: {}", rev_reg_id, json, err);
                    None
                }
            }
        }
        Err(err) => {
            warn!("Unable to get rev_reg_delta cache for rev_reg_id: {}, error: {}", rev_reg_id, err);
            None
        }
    }
}

pub fn update_rev_reg_ids_cache(cred_def_id: &str, rev_reg_id: &str) -> VcxResult<()> {
    debug!("Setting rev_reg_ids cache for cred_def_id {}, rev_reg_id {}", cred_def_id, rev_reg_id);
    match get_rev_reg_ids_cache(cred_def_id) {
        Some(mut old_vec) => {
            old_vec.rev_reg_ids.push(String::from(rev_reg_id));
            match serde_json::to_string(&old_vec) {
                Ok(ser_new_vec) => set_rev_reg_ids_cache(cred_def_id, ser_new_vec.as_str()),
                Err(_) => Err(VcxError::from(VcxErrorKind::SerializationError))
            }
        }
        None => {
            match serde_json::to_string(&vec![rev_reg_id]) {
                Ok(ser_new_vec) => set_rev_reg_ids_cache(cred_def_id, ser_new_vec.as_str()),
                Err(_) => Err(VcxError::from(VcxErrorKind::SerializationError))
            }
        }
    }
}

///
///
/// Saves rev reg delta cache.
/// Errors are silently ignored.
///
/// # Arguments
/// `rev_reg_id`: revocation registry id.
/// `cache`: Cache object.
///
pub fn set_rev_reg_delta_cache(rev_reg_id: &str, cache: &str) -> VcxResult<()> {
    debug!("Setting rev_reg_delta_cache for rev_reg_id {}, cache {}", rev_reg_id, cache);
    match serde_json::to_string(cache) {
        Ok(json) => {
            let wallet_id = format!("{}{}", REV_REG_DELTA_CACHE_PREFIX, rev_reg_id);
            match update_record_value(CACHE_TYPE, &wallet_id, &json)
                .or(add_record(CACHE_TYPE, &wallet_id, &json, None)) {
                Ok(_) => Ok(()),
                Err(err) => Err(err)
            }
        }
        Err(_) => {
            Err(VcxError::from(VcxErrorKind::SerializationError))
        }
    }
}

///
///
/// Clears the cache
/// Errors are silently ignored.
///
/// # Arguments
/// `rev_reg_id`: revocation registry id.
/// `cache`: Cache object.
///
pub fn clear_rev_reg_delta_cache(rev_reg_id: &str) -> VcxResult<String> {
    debug!("Clearing rev_reg_delta_cache for rev_reg_id {}", rev_reg_id);
    if let Some(last_delta) = get_rev_reg_delta_cache(rev_reg_id) {
        debug!("Got last delta = {}", last_delta);
        let wallet_id = format!("{}{}", REV_REG_DELTA_CACHE_PREFIX, rev_reg_id);
        delete_record(CACHE_TYPE, &wallet_id)?;
        debug!("Record with id {} deleted", wallet_id);
        Ok(last_delta)
    } else {
        Err(VcxError::from(VcxErrorKind::IOError))
    }
}

pub fn get_from_cache<'a, T: serde::de::DeserializeOwned>(prefix: &str, id: &str) -> VcxResult<T> {
    let wallet_id = format!("{}:{}", prefix, id);
    match get_record(CACHE_TYPE, &wallet_id, &json!({"retrieveType": false, "retrieveValue": true, "retrieveTags": false}).to_string()) {
        Ok(json) => serde_json::from_str(&json)
            .and_then(|x: serde_json::Value| serde_json::from_str(x.get("value").unwrap_or(&serde_json::Value::Null).as_str().unwrap_or("")))
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Unable to deserialize object from cache, prefix: {}, id: {}, json: {}, error: {}", prefix, id, json, err))),
        Err(err) => Err(VcxError::from_msg(VcxErrorKind::WalletAccessFailed , format!("Unable to read object from wallet, prefix: {}, id: {}, error: {} ", prefix, id, err)))
    }
}

pub fn save_to_cache<T: serde::Serialize>(prefix: &str, id: &str, obj: &T) -> VcxResult<()> {
    match serde_json::to_string(obj) {
        Ok(json) => {
            let wallet_id = format!("{}:{}", prefix, id);
            update_record_value(CACHE_TYPE, &wallet_id, &json)
                .or(add_record(CACHE_TYPE, &wallet_id, &json, None))
        },
        Err(err) => Err(VcxError::from_msg(VcxErrorKind::WalletAccessFailed , format!("Unable to convert object in cache to JSON, error: {:?}", err)))
    }
}
