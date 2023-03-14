use indy_credx::Error as CredxError;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind};

impl From<CredxError> for AriesVcxError {
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
            indy_credx::ErrorKind::Input => AriesVcxError::from_msg(AriesVcxErrorKind::InvalidInput, err),
            indy_credx::ErrorKind::IOError => AriesVcxError::from_msg(AriesVcxErrorKind::IOError, err),
            indy_credx::ErrorKind::InvalidState => AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err),
            indy_credx::ErrorKind::Unexpected => AriesVcxError::from_msg(AriesVcxErrorKind::UnknownError, err),
            indy_credx::ErrorKind::CredentialRevoked => AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err),
            indy_credx::ErrorKind::InvalidUserRevocId => AriesVcxError::from_msg(AriesVcxErrorKind::InvalidInput, err),
            indy_credx::ErrorKind::ProofRejected => AriesVcxError::from_msg(AriesVcxErrorKind::ProofRejected, err),
            indy_credx::ErrorKind::RevocationRegistryFull => {
                AriesVcxError::from_msg(AriesVcxErrorKind::InvalidState, err)
            }
        }
    }
}
