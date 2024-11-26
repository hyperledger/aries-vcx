use did_resolver::{did_doc::schema::types::uri::UriWrapperError, did_parser_nom};
use thiserror::Error;

use super::DidCheqdError;

#[derive(Error, Debug)]
pub enum ParsingErrorSource {
    #[error("DID document parsing error: {0}")]
    DidDocumentParsingError(#[from] did_parser_nom::ParseError),
    #[error("DID document parsing URI error: {0}")]
    DidDocumentParsingUriError(#[from] UriWrapperError),
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Invalid URL: {0}")]
    UrlParsingError(url::ParseError),
    #[error("Invalid encoding: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Invalid encoding: {0}")]
    IntConversionError(#[from] std::num::TryFromIntError),
}

impl From<did_parser_nom::ParseError> for DidCheqdError {
    fn from(error: did_parser_nom::ParseError) -> Self {
        DidCheqdError::ParsingError(ParsingErrorSource::DidDocumentParsingError(error))
    }
}

impl From<UriWrapperError> for DidCheqdError {
    fn from(error: UriWrapperError) -> Self {
        DidCheqdError::ParsingError(ParsingErrorSource::DidDocumentParsingUriError(error))
    }
}

impl From<serde_json::Error> for DidCheqdError {
    fn from(error: serde_json::Error) -> Self {
        DidCheqdError::ParsingError(ParsingErrorSource::JsonError(error))
    }
}

impl From<url::ParseError> for DidCheqdError {
    fn from(error: url::ParseError) -> Self {
        DidCheqdError::ParsingError(ParsingErrorSource::UrlParsingError(error))
    }
}

impl From<std::string::FromUtf8Error> for DidCheqdError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        DidCheqdError::ParsingError(ParsingErrorSource::Utf8Error(error))
    }
}

impl From<std::num::TryFromIntError> for DidCheqdError {
    fn from(error: std::num::TryFromIntError) -> Self {
        DidCheqdError::ParsingError(ParsingErrorSource::IntConversionError(error))
    }
}
