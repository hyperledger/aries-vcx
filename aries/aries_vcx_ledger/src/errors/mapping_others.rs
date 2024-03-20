use std::sync::PoisonError;

use super::error::VcxLedgerError;

impl<T> From<PoisonError<T>> for VcxLedgerError {
    fn from(err: PoisonError<T>) -> Self {
        VcxLedgerError::InvalidState(err.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for VcxLedgerError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        VcxLedgerError::InvalidState(err.to_string())
    }
}
