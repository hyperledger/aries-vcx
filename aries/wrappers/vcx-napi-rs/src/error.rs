use libvcx_core::{
    aries_vcx::errors::error::AriesVcxError,
    errors::error::{LibvcxError, LibvcxErrorKind},
    serde_json::json,
};
use libvcx_logger::error::LibvcxLoggerError;
use thiserror::Error;

pub fn to_napi_err(err: LibvcxError) -> napi::Error {
    let reason = json!({
        "vcxErrKind": err.kind().to_string(),
        "vcxErrCode": u32::from(err.kind()),
        "vcxErrMessage": err.msg,
    })
    .to_string();
    napi::Error::new(
        napi::Status::GenericFailure,
        format!("vcx_err_json:{reason}"),
    )
}

pub fn ariesvcx_to_napi_err(err: AriesVcxError) -> napi::Error {
    to_napi_err(LibvcxError::from(err))
}

#[derive(Debug, Error)]
pub enum NapiError {
    #[error("{0}")]
    LoggingError(#[from] LibvcxLoggerError),
}

impl From<NapiError> for LibvcxError {
    fn from(value: NapiError) -> Self {
        LibvcxError::from_msg(LibvcxErrorKind::LoggingError, value)
    }
}
