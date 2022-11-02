use serde_json::{from_slice, Value};
use vdrtools_sys::WalletHandle;
use vdrtoolsrs::crypto;

use crate::error::prelude::*;

pub async fn unpack(wallet_handle: WalletHandle, payload: Vec<u8>) -> VcxResult<String> {
    trace!("unpack >>> processing payload of {} bytes", payload.len());

    let unpacked_msg = crypto::unpack_message(wallet_handle, &payload).await?;

    let msg = from_slice::<Value>(unpacked_msg.as_slice())
        .map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::InvalidJson,
                format!("Cannot deserialize message: {}", err),
            )
        })?
        .as_str()
        .ok_or(VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            "Cannot convert message value to string",
        ))?
        .to_string();

    trace!("unpack >>> msg: {:?}", msg);

    Ok(msg)
}
