use crate::errors::error::AriesVcxError;

pub mod formats;
pub mod holder;
mod holder_demos;
pub mod issuer;
mod issuer_demos;

// TODO - better name?

pub struct RecoveredSMError<T> {
    pub error: AriesVcxError,
    pub state_machine: T,
}

impl<T> std::fmt::Debug for RecoveredSMError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecoveredSMError")
            .field("error", &self.error)
            .finish()
    }
}

// TODO - impl Error for RecoveredSMError?

type VcxSMTransitionResult<NEW, OLD> = Result<NEW, RecoveredSMError<OLD>>;
