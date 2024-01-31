use anoncreds::{Error as AnoncredsError, ErrorKind};

use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};

impl From<AnoncredsError> for AriesVcxCoreError {
    fn from(err: AnoncredsError) -> Self {
        match err.kind() {
            ErrorKind::Input => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err)
            }
            ErrorKind::IOError => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::IOError, err),
            ErrorKind::InvalidState => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidState, err)
            }
            ErrorKind::Unexpected => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::UnknownError, err)
            }
            ErrorKind::CredentialRevoked => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidState, err)
            }
            ErrorKind::InvalidUserRevocId => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err)
            }
            ErrorKind::ProofRejected => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::ProofRejected, err)
            }
            ErrorKind::RevocationRegistryFull => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidState, err)
            }
        }
    }
}
