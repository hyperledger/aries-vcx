use libvcx_core::{
    aries_vcx::errors::error::AriesVcxError, errors::error::LibvcxError, serde_json::json,
};

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
