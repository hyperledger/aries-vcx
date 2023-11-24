use aries_askar::ErrorKind;

use super::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};

impl From<aries_askar::Error> for AriesVcxCoreError {
    fn from(err: aries_askar::Error) -> Self {
        match err.kind() {
            ErrorKind::Backend => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::WalletError, err)
            }
            ErrorKind::Busy => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::WalletError, err),
            ErrorKind::Custom => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::WalletError, err)
            }
            ErrorKind::Duplicate => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::DuplicationWalletRecord, err)
            }
            ErrorKind::Encryption => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::WalletError, err)
            }
            ErrorKind::Input => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::WalletError, err)
            }
            ErrorKind::NotFound => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::WalletRecordNotFound, err)
            }
            ErrorKind::Unexpected => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::WalletError, err)
            }
            ErrorKind::Unsupported => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::WalletError, err)
            }
        }
    }
}
