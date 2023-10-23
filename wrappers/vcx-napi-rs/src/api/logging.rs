use libvcx_core::errors::error::LibvcxError;
use napi_derive::napi;
use test_utils::test_logger::LibvcxDefaultLogger;

use crate::error::to_napi_err;

#[napi]
pub fn init_default_logger(pattern: Option<String>) -> napi::Result<()> {
    LibvcxDefaultLogger::init(pattern)
        .map_err(LibvcxError::from)
        .map_err(to_napi_err)
}
