use thiserror::Error;

pub type DidCheqdResult<T> = Result<T, DidCheqdError>;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DidCheqdError {
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
