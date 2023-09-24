use libvcx_core::api_vcx::api_global::settings;
use napi_derive::napi;

use crate::error::to_napi_err;

#[napi]
pub fn enable_mocks() -> ::napi::Result<()> {
    settings::enable_mocks().map_err(to_napi_err)?;
    Ok(())
}
