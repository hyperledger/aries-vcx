use libvcx_core::aries_vcx::messages::protocols::out_of_band::handshake_reuse::OutOfBandHandshakeReuse;
use libvcx_core::aries_vcx::protocols::oob::build_handshake_reuse_accepted_msg;
use libvcx_core::errors::error::{LibvcxError, LibvcxErrorKind};
use libvcx_core::serde_json;
use napi_derive::napi;

use crate::error::to_napi_err;

#[napi]
pub fn out_of_band_build_handshake_reuse_accepted_msg(handshake_reuse: String) -> napi::Result<String> {
    let handshake_reuse = serde_json::from_str::<OutOfBandHandshakeReuse>(&handshake_reuse)
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidJson,
                format!("Cannot deserialize handshake reuse: {:?}", err),
            )
        })
        .map_err(to_napi_err)?;
    Ok(serde_json::json!(build_handshake_reuse_accepted_msg(&handshake_reuse)
        .map_err(|err| err.into())
        .map_err(to_napi_err)?
        .to_a2a_message())
    .to_string())
}
