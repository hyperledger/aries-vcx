use std::sync::PoisonError;
use crate::api_lib::errors::error::{ErrorLibvcx, ErrorKindLibvcx};

impl<T> From<PoisonError<T>> for ErrorLibvcx {
    fn from(err: PoisonError<T>) -> Self {
        ErrorLibvcx::from_msg(ErrorKindLibvcx::PoisonedLock, err.to_string())
    }
}
