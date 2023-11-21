pub mod parsing;

use hyper::StatusCode;
use thiserror::Error;

use self::parsing::ParsingErrorSource;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DidWebError {
    #[error("DID method not supported: {0}")]
    MethodNotSupported(String),
    #[error("Representation not supported: {0}")]
    RepresentationNotSupported(String),
    #[error("Invalid DID: {0}")]
    InvalidDid(String),
    #[error("Parsing error: {0}")]
    ParsingError(#[from] ParsingErrorSource),
    #[error("URL parsing error: {0}")]
    HttpError(#[from] hyper::Error),
    #[error("Non-success server response: {0}")]
    NonSuccessResponse(StatusCode),
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
