use thiserror::Error;

use super::DidWebError;

#[derive(Error, Debug)]
pub enum ParsingErrorSource {
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Invalid encoding: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

impl From<serde_json::Error> for DidWebError {
    fn from(error: serde_json::Error) -> Self {
        DidWebError::ParsingError(ParsingErrorSource::JsonError(error))
    }
}

impl From<std::string::FromUtf8Error> for DidWebError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        DidWebError::ParsingError(ParsingErrorSource::Utf8Error(error))
    }
}
