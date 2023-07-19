use napi_derive::napi;

use libvcx_core::api_vcx::api_global::state::state_vcx_shutdown;
use libvcx_core::api_vcx::api_global::VERSION_STRING;

#[napi]
pub fn shutdown(_delete_all: Option<bool>) -> ::napi::Result<()> {
    state_vcx_shutdown();
    Ok(())
}

#[napi]
pub fn get_version() -> ::napi::Result<String> {
    Ok(VERSION_STRING.clone())
}
