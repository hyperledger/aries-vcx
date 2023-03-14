use vdrtools::{Locator, WalletHandle};

use crate::{errors::error::prelude::*, global::settings};

pub async fn sign(wallet_handle: WalletHandle, my_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() {
        return Ok(Vec::from(msg));
    }

    let res = Locator::instance()
        .crypto_controller
        .crypto_sign(wallet_handle, my_vk, msg)
        .await?;

    Ok(res)
}

pub async fn verify(vk: &str, msg: &[u8], signature: &[u8]) -> VcxResult<bool> {
    if settings::indy_mocks_enabled() {
        return Ok(true);
    }

    let res = Locator::instance()
        .crypto_controller
        .crypto_verify(vk, msg, signature)
        .await?;

    Ok(res)
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

    // parse json array of keys
    let receiver_list = serde_json::from_str::<Vec<String>>(receiver_keys)
        .map_err(|_| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidJson, "Invalid RecipientKeys has been passed"))
        .and_then(|list| {
            // break early and error out if no receivers keys are provided
            if list.is_empty() {
                Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidLibindyParam,
                    "Empty RecipientKeys has been passed",
                ))
            } else {
                Ok(list)
            }
        })?;

    let res = Locator::instance()
        .crypto_controller
        .pack_msg(
            msg.into(),
            receiver_list,
            sender_vk.map(ToOwned::to_owned),
            wallet_handle,
        )
        .await?;

    Ok(res)
}

pub async fn unpack_message(wallet_handle: WalletHandle, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() {
        return Ok(Vec::from(msg));
    }

    let res = Locator::instance()
        .crypto_controller
        .unpack_msg(serde_json::from_slice(msg)?, wallet_handle)
        .await?;

    Ok(res)
}

#[cfg(feature = "test_utils")]
pub async fn create_key(wallet_handle: WalletHandle, seed: Option<&str>) -> VcxResult<String> {
    use vdrtools::KeyInfo;

    let res = Locator::instance()
        .crypto_controller
        .create_key(
            wallet_handle,
            &KeyInfo {
                seed: seed.map(ToOwned::to_owned),
                crypto_type: None,
            },
        )
        .await?;

    Ok(res)
}
