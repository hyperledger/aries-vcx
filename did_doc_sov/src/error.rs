use thiserror::Error;

#[derive(Debug, Error)]
pub enum DidDocumentSovError {
    #[error("Attempted to access empty collection: {0}")]
    EmptyCollection(&'static str),
    #[error("DID document builder error: {0}")]
    DidDocumentBuilderError(#[from] did_doc::error::DidDocumentBuilderError),
    #[error("Unexpected service type: {0}")]
    UnexpectedServiceType(String),
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(usize),
}
