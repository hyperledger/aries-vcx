use thiserror::Error;

use crate::error::DidDocumentBuilderError;

#[derive(Debug, Error)]
pub enum DidDocumentSovError {
    #[error("Attempted to access empty collection: {0}")]
    EmptyCollection(&'static str),
    #[error("DID document builder error: {0}")]
    DidDocumentBuilderError(#[from] DidDocumentBuilderError),
    #[error("Unexpected service type: {0}")]
    UnexpectedServiceType(String),
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(usize),
    #[error("JSON error")]
    JsonError(#[from] serde_json::Error),
    #[error("Parsing err {0}")]
    ParsingError(String),
}
