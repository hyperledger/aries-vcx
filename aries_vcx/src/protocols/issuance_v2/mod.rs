use crate::errors::error::AriesVcxError;

pub mod formats;
pub mod holder;
pub mod issuer;
mod demos;

mod messages {
    #[derive(Clone)]
    pub struct ProposeCredentialV2;
    #[derive(Clone)]
    pub struct OfferCredentialV2;
    #[derive(Clone)]
    pub struct RequestCredentialV2;
    #[derive(Clone)]
    pub struct IssueCredentialV2;
}

// TODO - better name?

pub struct RecoveredSMError<T> {
    pub error: AriesVcxError,
    pub state_machine: T
}

impl<T> std::fmt::Debug for RecoveredSMError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecoveredSMError").field("error", &self.error).finish()
    }
}

// TODO - impl Error for RecoveredSMError?

type VcxRecoverableSMResult<T, SM> = Result<T, RecoveredSMError<SM>>;
