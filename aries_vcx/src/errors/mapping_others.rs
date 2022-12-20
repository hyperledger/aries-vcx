use crate::error::{VcxError, VcxErrorKind};
use crate::protocols::revocation_notification::sender::state_machine::SenderConfigBuilderError;
use std::sync;

impl From<SenderConfigBuilderError> for VcxError {
    fn from(err: SenderConfigBuilderError) -> VcxError {
        let vcx_error_kind = VcxErrorKind::InvalidConfiguration;
        VcxError::from_msg(vcx_error_kind, err.to_string())
    }
}

impl From<serde_json::Error> for VcxError {
    fn from(_err: serde_json::Error) -> Self {
        VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Invalid json"))
    }
}
