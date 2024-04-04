use aries_vcx::{
    aries_vcx_core::errors::error::AriesVcxCoreError,
    aries_vcx_wallet::errors::error::VcxWalletError, errors::error::AriesVcxError,
};

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

impl From<VcxWalletError> for VcxUniFFIError {
    fn from(e: VcxWalletError) -> Self {
        let default = e;
        VcxUniFFIError::AriesVcxWalletError {
            error_msg: format!("AriesVcxWalletError: {default}"),
        }
    }
}
