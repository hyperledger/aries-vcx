use vdrtools::{WalletHandle, Locator};

use crate::error::AgencyClientResult;
use crate::testing::mocking::agency_mocks_enabled;

pub async fn pack_message(
    wallet_handle: WalletHandle,
    sender_vk: Option<&str>,
    receiver_keys: &str,
    msg: &[u8],
) -> AgencyClientResult<Vec<u8>> {
    trace!(
        "pack_message >>> sender_vk: {:?}, receiver_keys: {}, msg: ...",
        sender_vk,
        receiver_keys
    );

    if agency_mocks_enabled() {
        trace!("pack_message >>> mocks enabled, returning message");
        return Ok(msg.to_vec());
    }

    let res = Locator::instance()
        .crypto_controller
        .pack_msg(
            msg.into(),
            serde_json::from_str::<Vec<String>>(receiver_keys)?,
            sender_vk.map(|s| s.into()),
            wallet_handle,
        ).await?;

    Ok(res)
}

pub async fn unpack_message(wallet_handle: WalletHandle, msg: &[u8]) -> AgencyClientResult<Vec<u8>> {
    if agency_mocks_enabled() {
        trace!("unpack_message >>> mocks enabled, returning message");
        return Ok(msg.to_vec());
    }

    let res = Locator::instance()
        .crypto_controller
        .unpack_msg(
            serde_json::from_slice(msg)?,
            wallet_handle,
        ).await?;

    Ok(res)
}
