use napi_derive::napi;

use libvcx_core::api_vcx::api_global::state::state_vcx_shutdown;
use libvcx_core::api_vcx::api_global::VERSION_STRING;

#[napi]
pub fn shutdown(delete_all: Option<bool>) -> ::napi::Result<()> {
    state_vcx_shutdown(delete_all.unwrap_or(false));
    Ok(())
}

#[napi]
pub fn get_version() -> ::napi::Result<String> {
    Ok(VERSION_STRING.clone())
}
