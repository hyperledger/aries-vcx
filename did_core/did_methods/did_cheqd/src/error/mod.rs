use parsing::ParsingErrorSource;
use thiserror::Error;

pub mod parsing;

pub type DidCheqdResult<T> = Result<T, DidCheqdError>;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DidCheqdError {
    #[error("DID method not supported: {0}")]
    MethodNotSupported(String),
    #[error("Cheqd network not supported: {0}")]
    NetworkNotSupported(String),
    #[error("Bad configuration: {0}")]
    BadConfiguration(String),
    #[error("Transport error: {0}")]
    TransportError(#[from] tonic::transport::Error),
    #[error("Non-success resolver response: {0}")]
    NonSuccessResponse(#[from] tonic::Status),
    #[error("Response from resolver is invalid: {0}")]
    InvalidResponse(String),
    #[error("Invalid DID Document structure resolved: {0}")]
    InvalidDidDocument(String),
    #[error("Invalid DID Url: {0}")]
    InvalidDidUrl(String),
    #[error("Resource could not be found: {0}")]
    ResourceNotFound(String),
    #[error("Parsing error: {0}")]
    ParsingError(#[from] ParsingErrorSource),
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
