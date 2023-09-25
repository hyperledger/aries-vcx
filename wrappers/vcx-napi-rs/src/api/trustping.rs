use libvcx_core::{
    aries_vcx::{messages::msg_fields::protocols::trust_ping::ping::Ping, protocols::trustping},
    errors::error::{LibvcxError, LibvcxErrorKind},
    serde_json,
};
use napi_derive::napi;

use crate::error::to_napi_err;

#[napi]
fn trustping_build_response_msg(ping: String) -> napi::Result<String> {
    let ping = serde_json::from_str::<Ping>(&ping)
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidJson,
                format!("Cannot deserialize Ping: {:?}", err),
            )
        })
        .map_err(to_napi_err)?;
    Ok(serde_json::json!(trustping::build_ping_response_msg(&ping)).to_string())
}

#[napi]
fn trustping_build_ping(request_response: bool, comment: Option<String>) -> String {
    serde_json::json!(trustping::build_ping(request_response, comment)).to_string()
}
