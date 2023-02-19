use std::sync::PoisonError;

use super::error::VcxUniFFIError;

impl<T> From<PoisonError<T>> for VcxUniFFIError {
    fn from(e: PoisonError<T>) -> Self {
        VcxUniFFIError::InternalError {
            error_msg: e.to_string(),
        }
    }
}

impl From<serde_json::Error> for VcxUniFFIError {
    fn from(e: serde_json::Error) -> Self {
        VcxUniFFIError::SerializationError {
            error_msg: e.to_string(),
        }
    }
}
