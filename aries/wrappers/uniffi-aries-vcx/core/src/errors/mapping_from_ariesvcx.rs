use aries_vcx::{aries_vcx_core::errors::error::AriesVcxCoreError, errors::error::AriesVcxError};

use super::error::VcxUniFFIError;

impl From<AriesVcxError> for VcxUniFFIError {
    fn from(e: AriesVcxError) -> Self {
        let default = e;
        VcxUniFFIError::AriesVcxError {
            error_msg: format!("AriesVcxError: {default}"),
        }
    }
}

impl From<AriesVcxCoreError> for VcxUniFFIError {
    fn from(e: AriesVcxCoreError) -> Self {
        let default = e;
        VcxUniFFIError::AriesVcxError {
            error_msg: format!("AriesVcxCoreError: {default}"),
        }
    }
}
