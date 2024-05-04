use indy_credx::Error as CredxError;

use super::error::VcxAnoncredsError;

impl From<CredxError> for VcxAnoncredsError {
    fn from(err: CredxError) -> Self {
        // Credx will occasionally wrap the real error within the `cause` of an ErrorKind::Input
        // error type So we use this cause error if the cause exists and can be downcast to
        // an credxerror
        let cause = if err.kind() == indy_credx::ErrorKind::Input {
            err.cause
                .as_ref()
                .and_then(|x| x.downcast_ref::<CredxError>())
        } else {
            None
        };
        let e = cause.unwrap_or(&err);

        match e.kind() {
            indy_credx::ErrorKind::Input | indy_credx::ErrorKind::InvalidUserRevocId => {
                VcxAnoncredsError::InvalidInput(err.to_string())
            }
            indy_credx::ErrorKind::IOError => VcxAnoncredsError::IOError(err.to_string()),
            indy_credx::ErrorKind::InvalidState
            | indy_credx::ErrorKind::RevocationRegistryFull
            | indy_credx::ErrorKind::CredentialRevoked => {
                VcxAnoncredsError::InvalidState(err.to_string())
            }
            indy_credx::ErrorKind::Unexpected => VcxAnoncredsError::UnknownError(err.to_string()),
            indy_credx::ErrorKind::ProofRejected => {
                VcxAnoncredsError::ProofRejected(err.to_string())
            }
        }
    }
}
