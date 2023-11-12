use libvcx_core::{
    aries_vcx::{
        messages::{msg_fields::protocols::out_of_band::reuse::HandshakeReuse, AriesMessage},
        protocols::oob::build_handshake_reuse_accepted_msg,
    },
    errors::error::{LibvcxError, LibvcxErrorKind},
    serde_json,
};
use napi_derive::napi;

use crate::error::to_napi_err;

#[napi]
pub fn out_of_band_build_handshake_reuse_accepted_msg(
    handshake_reuse: String,
) -> napi::Result<String> {
    let handshake_reuse = serde_json::from_str::<HandshakeReuse>(&handshake_reuse)
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidJson,
                format!("Cannot deserialize handshake reuse: {:?}", err),
            )
        })
        .map_err(to_napi_err)?;
    Ok(serde_json::json!(AriesMessage::from(
        build_handshake_reuse_accepted_msg(&handshake_reuse)
            .map_err(|err| err.into())
            .map_err(to_napi_err)?
    ))
    .to_string())
}
