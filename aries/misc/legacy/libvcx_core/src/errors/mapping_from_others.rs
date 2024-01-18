use std::sync::PoisonError;

use public_key::PublicKeyError;

use crate::errors::error::{LibvcxError, LibvcxErrorKind};

impl<T> From<PoisonError<T>> for LibvcxError {
    fn from(err: PoisonError<T>) -> Self {
        LibvcxError::from_msg(LibvcxErrorKind::PoisonedLock, err.to_string())
    }
}

impl From<serde_json::Error> for LibvcxError {
    fn from(_err: serde_json::Error) -> Self {
        LibvcxError::from_msg(LibvcxErrorKind::InvalidJson, "Invalid json".to_string())
    }
}

impl From<PublicKeyError> for LibvcxError {
    fn from(value: PublicKeyError) -> Self {
        LibvcxError::from_msg(LibvcxErrorKind::InvalidVerkey, value)
    }
}
