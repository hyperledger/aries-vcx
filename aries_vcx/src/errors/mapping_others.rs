use std::sync::PoisonError;

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::revocation_notification::sender::state_machine::SenderConfigBuilderError,
};

impl From<SenderConfigBuilderError> for AriesVcxError {
    fn from(err: SenderConfigBuilderError) -> AriesVcxError {
        let vcx_error_kind = AriesVcxErrorKind::InvalidConfiguration;
        AriesVcxError::from_msg(vcx_error_kind, err.to_string())
    }
}

impl From<serde_json::Error> for AriesVcxError {
    fn from(_err: serde_json::Error) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidJson, "Invalid json".to_string())
    }
}

impl<T> From<PoisonError<T>> for AriesVcxError {
    fn from(err: PoisonError<T>) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err.to_string())
    }
}
