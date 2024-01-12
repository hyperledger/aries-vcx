use libvcx_core::errors::error::LibvcxError;
use libvcx_logger::LibvcxDefaultLogger;
use napi_derive::napi;

use crate::error::{to_napi_err, NapiError};

#[napi]
pub fn init_default_logger(pattern: Option<String>) -> napi::Result<()> {
    LibvcxDefaultLogger::init(pattern)
        .map_err(NapiError::from)
        .map_err(LibvcxError::from)
        .map_err(to_napi_err)
}
