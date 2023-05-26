use crate::errors::error::{LibvcxError, LibvcxErrorKind};
use std::sync::PoisonError;

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

impl From<did_doc::error::DidDocumentBuilderError> for LibvcxError {
    fn from(err: did_doc::error::DidDocumentBuilderError) -> Self {
        LibvcxError::from_msg(LibvcxErrorKind::DidDocumentError, err.to_string())
    }
}

impl From<diddoc_legacy::errors::error::DiddocError> for LibvcxError {
    fn from(err: diddoc_legacy::errors::error::DiddocError) -> Self {
        LibvcxError::from_msg(LibvcxErrorKind::DidDocumentError, err.to_string())
    }
}
