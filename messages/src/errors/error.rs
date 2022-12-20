use std::fmt;
use std::error::Error;

pub mod prelude {
    pub use crate::errors::error::{err_msg, ErrorMessages, ErrorKindMessages, MessagesResult};
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, thiserror::Error)]
pub enum ErrorKindMessages {
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
pub struct ErrorMessages {
    msg: String,
    kind: ErrorKindMessages,
}

impl fmt::Display for ErrorMessages {
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

impl ErrorMessages {
    pub fn from_msg<D>(kind: ErrorKindMessages, msg: D) -> ErrorMessages
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        ErrorMessages {
            msg: msg.to_string(),
            kind,
        }
    }

    pub fn kind(&self) -> ErrorKindMessages {
        self.kind
    }
}

pub fn err_msg<D>(kind: ErrorKindMessages, msg: D) -> ErrorMessages
where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    ErrorMessages::from_msg(kind, msg)
}

pub type MessagesResult<T> = Result<T, ErrorMessages>;
