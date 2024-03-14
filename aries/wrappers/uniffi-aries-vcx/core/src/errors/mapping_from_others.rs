use std::{string::FromUtf8Error, sync::PoisonError};

use aries_vcx::aries_vcx_wallet::errors::error::VcxWalletError;

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

impl From<FromUtf8Error> for VcxUniFFIError {
    fn from(e: FromUtf8Error) -> Self {
        VcxUniFFIError::StringParseError {
            error_msg: e.to_string(),
        }
    }
}

impl From<did_parser::ParseError> for VcxUniFFIError {
    fn from(e: did_parser::ParseError) -> Self {
        VcxUniFFIError::StringParseError {
            error_msg: e.to_string(),
        }
    }
}

impl From<VcxWalletError> for VcxUniFFIError {
    fn from(value: VcxWalletError) -> Self {
        Self::AriesVcxWalletError {
            error_msg: value.to_string(),
        }
    }
}
