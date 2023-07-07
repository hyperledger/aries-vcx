use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};
use anoncreds::{Error as AnoncredsError, ErrorKind};

impl From<AnoncredsError> for AriesVcxCoreError {
    fn from(err: AnoncredsError) -> Self {
        let cause = if err.kind() == ErrorKind::Input {
            err.cause.as_ref().and_then(|x| x.downcast_ref::<AnoncredsError>())
        } else {
            None
        };
        let e = cause.unwrap_or(&err);

        match e.kind() {
            ErrorKind::Input => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err),
            ErrorKind::IOError => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::IOError, err),
            ErrorKind::InvalidState => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidState, err),
            ErrorKind::Unexpected => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::UnknownError, err),
            ErrorKind::CredentialRevoked => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidState, err),
            ErrorKind::InvalidUserRevocId => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err),
            ErrorKind::ProofRejected => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::ProofRejected, err),
            ErrorKind::RevocationRegistryFull => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidState, err),
        }
    }
}
