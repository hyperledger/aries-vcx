use std::{error::Error, fmt};

pub mod prelude {
    pub use crate::errors::error::{err_msg, DiddocError, DiddocErrorKind, DiddocResult};
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, thiserror::Error)]
pub enum DiddocErrorKind {
    #[error("Object is in invalid state for requested operation")]
    InvalidState,
    #[error("Invalid JSON string")]
    InvalidJson,
    #[error("Unable to serialize")]
    SerializationError,
    #[error("Invalid URL")]
    InvalidUrl,
    // todo: reduce granularity - just funnel the 3 errs below into single "ValidationError"
    #[error("Invalid DID")]
    InvalidDid,
    #[error("Invalid VERKEY")]
    InvalidVerkey,
    #[error("Value needs to be base58")]
    NotBase58,
}

#[derive(Debug, thiserror::Error)]
pub struct DiddocError {
    msg: String,
    kind: DiddocErrorKind,
}

impl fmt::Display for DiddocError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Error: {}\n", self.msg)?;
        let mut current = self.source();
        while let Some(cause) = current {
            writeln!(f, "Caused by:\n\t{}", cause)?;
            current = cause.source();
        }
        Ok(())
    }
}

impl DiddocError {
    pub fn from_msg<D>(kind: DiddocErrorKind, msg: D) -> DiddocError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        DiddocError {
            msg: msg.to_string(),
            kind,
        }
    }

    pub fn kind(&self) -> DiddocErrorKind {
        self.kind
    }
}

pub fn err_msg<D>(kind: DiddocErrorKind, msg: D) -> DiddocError
where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    DiddocError::from_msg(kind, msg)
}

pub type DiddocResult<T> = Result<T, DiddocError>;
