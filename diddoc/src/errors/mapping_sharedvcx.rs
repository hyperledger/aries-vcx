use shared_vcx::errors::error::{SharedVcxError, SharedVcxErrorKind};

use crate::errors::error::{DiddocError, DiddocErrorKind};

impl From<SharedVcxErrorKind> for DiddocErrorKind {
    fn from(error: SharedVcxErrorKind) -> Self {
        match error {
            SharedVcxErrorKind::InvalidDid => DiddocErrorKind::InvalidDid,
            SharedVcxErrorKind::InvalidVerkey => DiddocErrorKind::InvalidVerkey,
            SharedVcxErrorKind::NotBase58 => DiddocErrorKind::NotBase58,
            SharedVcxErrorKind::InvalidState => DiddocErrorKind::InvalidDid,
            SharedVcxErrorKind::InvalidConfiguration => DiddocErrorKind::InvalidConfiguration,
            SharedVcxErrorKind::PostMessageFailed => DiddocErrorKind::PostMessageFailed
        }
    }
}

impl From<SharedVcxError> for DiddocError {
    fn from(error: SharedVcxError) -> Self {
        let kind: DiddocErrorKind = error.kind().into();
        DiddocError::from_msg(kind, error.to_string())
    }
}
