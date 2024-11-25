use parsing::ParsingErrorSource;
use thiserror::Error;

pub mod parsing;

pub type DidCheqdResult<T> = Result<T, DidCheqdError>;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DidCheqdError {
    #[error("Parsing error: {0}")]
    ParsingError(#[from] ParsingErrorSource),
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
