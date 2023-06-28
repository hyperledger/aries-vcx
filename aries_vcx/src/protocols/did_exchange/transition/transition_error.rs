use crate::errors::error::AriesVcxError;

#[derive(Debug)]
pub struct TransitionError<T> {
    pub state: T,
    pub error: AriesVcxError,
}

impl<T> std::fmt::Display for TransitionError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Transition error: {}", self.error)
    }
}

impl<T: std::fmt::Debug> std::error::Error for TransitionError<T> {}

impl<T> From<TransitionError<T>> for AriesVcxError {
    fn from(err: TransitionError<T>) -> Self {
        err.error
    }
}
