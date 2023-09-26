use messages::AriesMessage;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind};

pub mod formats;
pub mod holder;
pub mod issuer;

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

fn unmatched_thread_id_error(msg: AriesMessage, expected_thid: &str) -> AriesVcxError {
    AriesVcxError::from_msg(
        AriesVcxErrorKind::InvalidJson,
        format!(
            "Cannot handle message {:?}: thread id does not match, expected {:?}",
            msg, expected_thid
        ),
    )
}
