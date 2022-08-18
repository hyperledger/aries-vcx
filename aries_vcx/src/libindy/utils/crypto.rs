/* test isn't ready until > libindy 1.0.1 */
use indy::crypto;
use indy_sys::WalletHandle;

use crate::error::prelude::*;
use crate::global::settings;

pub async fn sign(wallet_handle: WalletHandle, my_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() {
        return Ok(Vec::from(msg));
    }

    crypto::sign(wallet_handle, my_vk, msg).await.map_err(VcxError::from)
}

pub async fn verify(vk: &str, msg: &[u8], signature: &[u8]) -> VcxResult<bool> {
    if settings::indy_mocks_enabled() {
        return Ok(true);
    }

    crypto::verify(vk, msg, signature).await.map_err(VcxError::from)
}

pub async fn pack_message(
    wallet_handle: WalletHandle,
    sender_vk: Option<&str>,
    receiver_keys: &str,
    msg: &[u8],
) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() {
        return Ok(msg.to_vec());
    }

    crypto::pack_message(wallet_handle, msg, receiver_keys, sender_vk)
        .await
        .map_err(VcxError::from)
}

pub async fn unpack_message(wallet_handle: WalletHandle, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() {
        return Ok(Vec::from(msg));
    }

    crypto::unpack_message(wallet_handle, msg).await.map_err(VcxError::from)
}

pub async fn create_key(wallet_handle: WalletHandle, seed: Option<&str>) -> VcxResult<String> {
    let key_json = json!({ "seed": seed }).to_string();

    crypto::create_key(wallet_handle, Some(&key_json))
        .await
        .map_err(VcxError::from)
}
