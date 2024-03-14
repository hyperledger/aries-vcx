use aries_askar::ErrorKind;

use super::error::{NotFoundInfo, VcxWalletError};

impl From<aries_askar::Error> for VcxWalletError {
    fn from(err: aries_askar::Error) -> Self {
        match err.kind() {
            ErrorKind::Backend
            | ErrorKind::Custom
            | ErrorKind::Encryption
            | ErrorKind::Input
            | ErrorKind::Unexpected
            | ErrorKind::Unsupported
            | ErrorKind::Busy => VcxWalletError::unknown_error(err),
            ErrorKind::Duplicate => VcxWalletError::DuplicateRecord(err.to_string()),
            ErrorKind::NotFound => {
                VcxWalletError::RecordNotFound(NotFoundInfo::new_without_details())
            }
        }
    }
}
