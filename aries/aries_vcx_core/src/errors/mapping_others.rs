use std::sync::PoisonError;

use did_parser::ParseError;

use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};

impl From<serde_json::Error> for AriesVcxCoreError {
    fn from(err: serde_json::Error) -> Self {
        AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::InvalidJson,
            format!("Invalid json: {err}"),
        )
    }
}

impl<T> From<PoisonError<T>> for AriesVcxCoreError {
    fn from(err: PoisonError<T>) -> Self {
        AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidState, err.to_string())
    }
}

impl From<ParseError> for AriesVcxCoreError {
    fn from(err: ParseError) -> Self {
        AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::ParsingError, err.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for AriesVcxCoreError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidState, err.to_string())
    }
}
