use crate::errors::error::{ErrorAriesVcx, ErrorKindAriesVcx};
use crate::protocols::revocation_notification::sender::state_machine::SenderConfigBuilderError;
use std::sync;

impl From<SenderConfigBuilderError> for ErrorAriesVcx {
    fn from(err: SenderConfigBuilderError) -> ErrorAriesVcx {
        let vcx_error_kind = ErrorKindAriesVcx::InvalidConfiguration;
        ErrorAriesVcx::from_msg(vcx_error_kind, err.to_string())
    }
}

impl From<serde_json::Error> for ErrorAriesVcx {
    fn from(_err: serde_json::Error) -> Self {
        ErrorAriesVcx::from_msg(ErrorKindAriesVcx::InvalidJson, format!("Invalid json"))
    }
}
