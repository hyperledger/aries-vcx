use napi_derive::napi;

use libvcx_core::aries_vcx::utils::test_logger::LibvcxDefaultLogger;

use crate::error::ariesvcx_to_napi_err;

#[napi]
pub fn init_default_logger(pattern: Option<String>) -> napi::Result<()> {
    LibvcxDefaultLogger::init(pattern).map_err(ariesvcx_to_napi_err)
}
