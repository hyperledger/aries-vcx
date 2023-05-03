use aries_vcx::{aries_vcx_core::errors::error::AriesVcxCoreError, errors::error::AriesVcxError};

use super::error::VcxUniFFIError;

impl From<AriesVcxError> for VcxUniFFIError {
    fn from(e: AriesVcxError) -> Self {
        match e {
            default => VcxUniFFIError::AriesVcxError {
                error_msg: default.to_string(),
            },
        }
    }
}

impl From<AriesVcxCoreError> for VcxUniFFIError {
    fn from(e: AriesVcxCoreError) -> Self {
        match e {
            default => VcxUniFFIError::AriesVcxError {
                error_msg: default.to_string(),
            },
        }
    }
}
