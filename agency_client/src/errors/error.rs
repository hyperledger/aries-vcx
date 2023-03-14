use std::{error::Error, fmt};

pub mod prelude {
    pub use crate::errors::error::*;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, thiserror::Error)]
pub enum AgencyClientErrorKind {
    // Common
    #[error("Object is in invalid state for requested operation")]
    InvalidState,
    #[error("Invalid Configuration")]
    InvalidConfiguration,
    #[error("Obj was not found with handle")]
    InvalidJson,
    #[error("Invalid Option")]
    InvalidOption,
    #[error("Invalid MessagePack")]
    InvalidMessagePack,
    #[error("IO Error, possibly creating a backup wallet")]
    IOError,

    #[error("Message failed in post")]
    PostMessageFailed,

    // Wallet
    #[error("Invalid Wallet or Search Handle")]
    InvalidWalletHandle,

    // Validation
    #[error("Unknown Error")]
    UnknownError,
    #[error("Invalid DID")]
    InvalidDid,
    #[error("Invalid VERKEY")]
    InvalidVerkey,
    #[error("Invalid URL")]
    InvalidUrl,
    #[error("Unable to serialize")]
    SerializationError,
    #[error("Value needs to be base58")]
    NotBase58,

    // A2A
    #[error("Invalid HTTP response.")]
    InvalidHttpResponse,
}

#[derive(Debug, thiserror::Error)]
pub struct AgencyClientError {
    msg: String,
    kind: AgencyClientErrorKind,
}

impl fmt::Display for AgencyClientError {
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

impl AgencyClientError {
    pub fn from_msg<D>(kind: AgencyClientErrorKind, msg: D) -> AgencyClientError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        AgencyClientError {
            msg: msg.to_string(),
            kind,
        }
    }

    pub fn kind(&self) -> AgencyClientErrorKind {
        self.kind
    }
}

pub type AgencyClientResult<T> = Result<T, AgencyClientError>;
