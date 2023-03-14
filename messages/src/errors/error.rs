use std::{error::Error, fmt};

pub mod prelude {
    pub use crate::errors::error::{err_msg, MessagesError, MessagesErrorKind, MessagesResult};
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, thiserror::Error)]
pub enum MessagesErrorKind {
    #[error("Object is in invalid state for requested operation")]
    InvalidState,
    #[error("Invalid JSON string")]
    InvalidJson,
    #[error("IO Error, possibly creating a backup wallet")]
    IOError,
    #[error("Unable to serialize")]
    SerializationError,
    #[error("Invalid DID")]
    InvalidDid,
    #[error("Invalid verkey")]
    InvalidVerkey,
    #[error("Invalid URL")]
    InvalidUrl,
    #[error("Value needs to be base58")]
    NotBase58,
}

#[derive(Debug, thiserror::Error)]
pub struct MessagesError {
    msg: String,
    kind: MessagesErrorKind,
}

impl fmt::Display for MessagesError {
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

impl MessagesError {
    pub fn from_msg<D>(kind: MessagesErrorKind, msg: D) -> MessagesError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        MessagesError {
            msg: msg.to_string(),
            kind,
        }
    }

    pub fn kind(&self) -> MessagesErrorKind {
        self.kind
    }
}

pub fn err_msg<D>(kind: MessagesErrorKind, msg: D) -> MessagesError
where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    MessagesError::from_msg(kind, msg)
}

pub type MessagesResult<T> = Result<T, MessagesError>;
