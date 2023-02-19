use aries_vcx::errors::error::AriesVcxError;

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
