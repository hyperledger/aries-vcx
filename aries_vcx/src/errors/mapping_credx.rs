use crate::errors::error::{AriesVcxError, AriesVcxErrorKind};
use indy_credx::Error as CredxError;

impl From<CredxError> for AriesVcxError {
    fn from(err: CredxError) -> Self {
        match err.kind() {
            indy_credx::ErrorKind::Input => AriesVcxError::from_msg(AriesVcxErrorKind::InvalidInput, err),
            indy_credx::ErrorKind::IOError => AriesVcxError::from_msg(AriesVcxErrorKind::IOError, err),
            indy_credx::ErrorKind::InvalidState => AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err),
            indy_credx::ErrorKind::Unexpected => AriesVcxError::from_msg(AriesVcxErrorKind::UnknownError, err),
            indy_credx::ErrorKind::CredentialRevoked => AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err),
            indy_credx::ErrorKind::InvalidUserRevocId => AriesVcxError::from_msg(AriesVcxErrorKind::InvalidInput, err),
            indy_credx::ErrorKind::ProofRejected => {
                AriesVcxError::from_msg(AriesVcxErrorKind::ProofRejected, err)
            }
            indy_credx::ErrorKind::RevocationRegistryFull => {
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err)
            }
        }
    }
}
