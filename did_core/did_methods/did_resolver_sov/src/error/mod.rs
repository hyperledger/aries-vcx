pub mod parsing;
mod resolution;

use aries_vcx_core::errors::error::AriesVcxCoreError;
use did_resolver::did_doc::{error::DidDocumentBuilderError, schema::types::uri::UriWrapperError};
use thiserror::Error;

use self::parsing::ParsingErrorSource;
use crate::error::DidSovError::ParsingError;

// TODO: DIDDocumentBuilderError should do key validation and the error
// should me mapped accordingly
// TODO: Perhaps split into input errors and external errors?
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DidSovError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("DID method not supported: {0}")]
    MethodNotSupported(String),
    #[error("Representation not supported: {0}")]
    RepresentationNotSupported(String),
    #[error("Internal error")]
    InternalError,
    #[error("Invalid DID: {0}")]
    InvalidDid(String),
    #[error("AriesVCX Core error: {0}")]
    AriesVcxCoreError(#[from] AriesVcxCoreError),
    #[error("DID Document Builder Error: {0}")]
    DidDocumentBuilderError(#[from] DidDocumentBuilderError),
    #[error("Parsing error: {0}")]
    ParsingError(#[from] ParsingErrorSource),
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl From<UriWrapperError> for DidSovError {
    fn from(error: UriWrapperError) -> Self {
        ParsingError(ParsingErrorSource::DidDocumentParsingUriError(error))
    }
}
