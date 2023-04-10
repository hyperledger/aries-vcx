use std::sync::PoisonError;

use aries_vcx_core::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind};
use crate::protocols::revocation_notification::sender::state_machine::SenderConfigBuilderError;

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

// TODO
impl From<AriesVcxCoreError> for AriesVcxError {
    fn from(err: AriesVcxCoreError) -> Self {
        AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err.to_string())
    }
}

// TODO
impl From<AriesVcxError> for AriesVcxCoreError {
    fn from(err: AriesVcxError) -> Self {
        AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidState, err.to_string())
    }
}
