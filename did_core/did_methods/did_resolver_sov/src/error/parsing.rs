use did_resolver::{did_doc::schema::types::uri::UriWrapperError, did_parser_nom};
use thiserror::Error;

use super::DidSovError;

#[derive(Error, Debug)]
pub enum ParsingErrorSource {
    #[error("DID document parsing error: {0}")]
    DidDocumentParsingError(#[from] did_parser_nom::ParseError),
    #[error("DID document parsing URI error: {0}")]
    DidDocumentParsingUriError(#[from] UriWrapperError),
    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Ledger response parsing error: {0}")]
    LedgerResponseParsingError(String),
}

impl From<did_parser_nom::ParseError> for DidSovError {
    fn from(error: did_parser_nom::ParseError) -> Self {
        DidSovError::ParsingError(ParsingErrorSource::DidDocumentParsingError(error))
    }
}

impl From<serde_json::Error> for DidSovError {
    fn from(error: serde_json::Error) -> Self {
        DidSovError::ParsingError(ParsingErrorSource::SerdeError(error))
    }
}
