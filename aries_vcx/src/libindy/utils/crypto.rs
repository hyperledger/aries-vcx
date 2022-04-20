/* test isn't ready until > libindy 1.0.1 */
use indy::crypto;

use crate::{libindy, settings};
use crate::error::prelude::*;

pub async fn sign(my_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() { return Ok(Vec::from(msg).to_owned()); }

    crypto::sign(libindy::utils::wallet::get_wallet_handle(), my_vk, msg)
        .await
        .map_err(VcxError::from)
}

pub async fn verify(vk: &str, msg: &[u8], signature: &[u8]) -> VcxResult<bool> {
    if settings::indy_mocks_enabled() { return Ok(true); }

    crypto::verify(vk, msg, signature)
        .await
        .map_err(VcxError::from)
}

pub async fn pack_message(sender_vk: Option<&str>, receiver_keys: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() { return Ok(msg.to_vec()); }

    crypto::pack_message(libindy::utils::wallet::get_wallet_handle(), msg, receiver_keys, sender_vk)
        .await
        .map_err(VcxError::from)
}

pub async fn unpack_message(msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() { return Ok(Vec::from(msg).to_owned()); }

    crypto::unpack_message(libindy::utils::wallet::get_wallet_handle(), msg)
        .await
        .map_err(VcxError::from)
}

pub async fn create_key(seed: Option<&str>) -> VcxResult<String> {
    let key_json = json!({"seed": seed}).to_string();

    crypto::create_key(libindy::utils::wallet::get_wallet_handle(), Some(&key_json))
        .await
        .map_err(VcxError::from)
}
