/* test isn't ready until > libindy 1.0.1 */
use indy::crypto;
use indy::future::Future;

use crate::{libindy, settings};
use crate::error::prelude::*;

pub fn sign(my_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() { return Ok(Vec::from(msg).to_owned()); }

    crypto::sign(libindy::utils::wallet::get_wallet_handle(), my_vk, msg)
        .wait()
        .map_err(VcxError::from)
}

pub fn verify(vk: &str, msg: &[u8], signature: &[u8]) -> VcxResult<bool> {
    if settings::indy_mocks_enabled() { return Ok(true); }

    crypto::verify(vk, msg, signature)
        .wait()
        .map_err(VcxError::from)
}

pub fn pack_message(sender_vk: Option<&str>, receiver_keys: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() { return Ok(msg.to_vec()); }

    crypto::pack_message(libindy::utils::wallet::get_wallet_handle(), msg, receiver_keys, sender_vk)
        .wait()
        .map_err(VcxError::from)
}

pub fn unpack_message(msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() { return Ok(Vec::from(msg).to_owned()); }

    crypto::unpack_message(libindy::utils::wallet::get_wallet_handle(), msg)
        .wait()
        .map_err(VcxError::from)
}

pub fn create_key(seed: Option<&str>) -> VcxResult<String> {
    let key_json = json!({"seed": seed}).to_string();

    crypto::create_key(libindy::utils::wallet::get_wallet_handle(), Some(&key_json))
        .wait()
        .map_err(VcxError::from)
}