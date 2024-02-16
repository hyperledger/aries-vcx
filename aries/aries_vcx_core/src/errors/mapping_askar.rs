use aries_askar::ErrorKind;

use super::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};

impl From<aries_askar::Error> for AriesVcxCoreError {
    fn from(err: aries_askar::Error) -> Self {
        match err.kind() {
            ErrorKind::Backend
            | ErrorKind::Custom
            | ErrorKind::Encryption
            | ErrorKind::Input
            | ErrorKind::Unexpected
            | ErrorKind::Unsupported
            | ErrorKind::Busy => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::WalletError, err)
            }
            ErrorKind::Duplicate => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::DuplicationWalletRecord, err)
            }
            ErrorKind::NotFound => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::WalletRecordNotFound, err)
            }
        }
    }
}
