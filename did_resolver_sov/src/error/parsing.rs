use did_resolver::did_parser;
use thiserror::Error;

use super::DIDSovError;

#[derive(Error, Debug)]
pub enum ParsingErrorSource {
    #[error("DID document parsing error: {0}")]
    DidDocumentParsingError(#[from] did_parser::ParseError),
    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Ledger response parsing error: {0}")]
    LedgerResponseParsingError(String),
}

impl From<did_parser::ParseError> for DIDSovError {
    fn from(error: did_parser::ParseError) -> Self {
        DIDSovError::ParsingError(ParsingErrorSource::DidDocumentParsingError(error))
    }
}

impl From<serde_json::Error> for DIDSovError {
    fn from(error: serde_json::Error) -> Self {
        DIDSovError::ParsingError(ParsingErrorSource::SerdeError(error))
    }
}
