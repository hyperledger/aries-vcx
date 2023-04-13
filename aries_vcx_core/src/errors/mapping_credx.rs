use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};
use indy_credx::Error as CredxError;

impl From<CredxError> for AriesVcxCoreError {
    fn from(err: CredxError) -> Self {
        // Credx will occasionally wrap the real error within the `cause` of an ErrorKind::Input error type
        // So we use this cause error if the cause exists and can be downcast to an credxerror
        let cause = if err.kind() == indy_credx::ErrorKind::Input {
            err.cause.as_ref().and_then(|x| x.downcast_ref::<CredxError>())
        } else {
            None
        };
        let e = cause.unwrap_or(&err);

        match e.kind() {
            indy_credx::ErrorKind::Input => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err),
            indy_credx::ErrorKind::IOError => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::IOError, err),
            indy_credx::ErrorKind::InvalidState => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidState, err)
            }
            indy_credx::ErrorKind::Unexpected => AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::UnknownError, err),
            indy_credx::ErrorKind::CredentialRevoked => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidState, err)
            }
            indy_credx::ErrorKind::InvalidUserRevocId => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err)
            }
            indy_credx::ErrorKind::ProofRejected => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::ProofRejected, err)
            }
            indy_credx::ErrorKind::RevocationRegistryFull => {
                AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidState, err)
            }
        }
    }
}
