use crate::errors::error::AriesVcxError;

#[derive(Debug)]
pub enum FailureReason {
    Error(AriesVcxError),
    OtherReason(String),
}

#[derive(Debug)]
pub struct Failed {
    pub reason: FailureReason,
}

impl Failed {
    pub fn from_error(error: AriesVcxError) -> Self {
        Failed {
            reason: FailureReason::Error(error),
        }
    }

    pub fn from_other_reason(other_reason: String) -> Self {
        Failed {
            reason: FailureReason::OtherReason(other_reason),
        }
    }
}
