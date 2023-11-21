use vdrtools::Locator;

use crate::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    WalletHandle,
};

pub async fn sign(wallet_handle: WalletHandle, my_vk: &str, msg: &[u8]) -> VcxCoreResult<Vec<u8>> {
    let res = Locator::instance()
        .crypto_controller
        .crypto_sign(wallet_handle, my_vk, msg)
        .await?;

    Ok(res)
}

pub async fn verify(vk: &str, msg: &[u8], signature: &[u8]) -> VcxCoreResult<bool> {
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
) -> VcxCoreResult<Vec<u8>> {
    // parse json array of keys
    let receiver_list = serde_json::from_str::<Vec<String>>(receiver_keys)
        .map_err(|_| {
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidJson,
                "Invalid RecipientKeys has been passed",
            )
        })
        .and_then(|list| {
            // break early and error out if no receivers keys are provided
            if list.is_empty() {
                Err(AriesVcxCoreError::from_msg(
                    AriesVcxCoreErrorKind::InvalidLibindyParam,
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

pub async fn unpack_message(wallet_handle: WalletHandle, msg: &[u8]) -> VcxCoreResult<Vec<u8>> {
    let res = Locator::instance()
        .crypto_controller
        .unpack_msg(serde_json::from_slice(msg)?, wallet_handle)
        .await?;

    Ok(res)
}

pub async fn create_key(wallet_handle: WalletHandle, seed: Option<&str>) -> VcxCoreResult<String> {
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
