use vdrtools_sys::WalletHandle;
use vdrtoolsrs::crypto;

use crate::error::prelude::*;

pub async fn unpack(wallet_handle: WalletHandle, payload: Vec<u8>) -> VcxResult<String> {
    trace!("unpack >>> processing payload of {} bytes", payload.len());

    let msg = String::from_utf8(
        crypto::unpack_message(wallet_handle, &payload)
            .await
            .map_err(|_| VcxError::from_msg(VcxErrorKind::InvalidMessagePack, "Failed to unpack message"))?,
    )
    .map_err(|_| {
        VcxError::from_msg(
            VcxErrorKind::InvalidMessageFormat,
            "Failed to convert message to utf8 string",
        )
    })?;

    trace!("unpack <<<");

    Ok(msg)
}
