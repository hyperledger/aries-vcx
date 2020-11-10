use indy::crypto;
use agency_comm::utils::error::prelude::*;
use futures::Future;

pub fn pack_message(sender_vk: Option<&str>, receiver_keys: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    // if settings::indy_mocks_enabled() { return Ok(msg.to_vec()); }

    crypto::pack_message(::libindy::utils::wallet::get_wallet_handle(), msg, receiver_keys, sender_vk)
        .wait()
        .map_err(VcxError::from)
}

pub fn unpack_message(msg: &[u8]) -> VcxResult<Vec<u8>> {
    // if settings::indy_mocks_enabled() { return Ok(Vec::from(msg).to_owned()); }

    crypto::unpack_message(::libindy::utils::wallet::get_wallet_handle(), msg)
        .wait()
        .map_err(VcxError::from)
}
