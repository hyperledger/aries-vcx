use crate::errors::error::{LibvcxError, LibvcxErrorKind};
use std::sync::PoisonError;

impl<T> From<PoisonError<T>> for LibvcxError {
    fn from(err: PoisonError<T>) -> Self {
        LibvcxError::from_msg(LibvcxErrorKind::PoisonedLock, err.to_string())
    }
}
