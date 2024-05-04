use anoncreds::{Error as AnoncredsError, ErrorKind};

use crate::errors::error::VcxAnoncredsError;

impl From<AnoncredsError> for VcxAnoncredsError {
    fn from(err: AnoncredsError) -> Self {
        match err.kind() {
            ErrorKind::Input => VcxAnoncredsError::InvalidInput(err.to_string()),
            ErrorKind::IOError => VcxAnoncredsError::IOError(err.to_string()),
            ErrorKind::InvalidState => VcxAnoncredsError::InvalidState(err.to_string()),
            ErrorKind::Unexpected => VcxAnoncredsError::UnknownError(err.to_string()),
            ErrorKind::CredentialRevoked => VcxAnoncredsError::InvalidState(err.to_string()),
            ErrorKind::InvalidUserRevocId => VcxAnoncredsError::InvalidInput(err.to_string()),
            ErrorKind::ProofRejected => VcxAnoncredsError::ProofRejected(err.to_string()),
            ErrorKind::RevocationRegistryFull => VcxAnoncredsError::InvalidState(err.to_string()),
        }
    }
}
