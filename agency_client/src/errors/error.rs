use std::fmt;
use std::error::Error;

pub mod prelude {
    pub use crate::errors::error::*;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, thiserror::Error)]
pub enum ErrorKindAgencyClient {
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
pub struct ErrorAgencyClient {
    msg: String,
    kind: ErrorKindAgencyClient,
}

impl fmt::Display for ErrorAgencyClient {
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

impl ErrorAgencyClient {
    pub fn from_msg<D>(kind: ErrorKindAgencyClient, msg: D) -> ErrorAgencyClient
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        ErrorAgencyClient {
            msg: msg.to_string(),
            kind,
        }
    }

    pub fn kind(&self) -> ErrorKindAgencyClient {
        self.kind
    }
}

pub type AgencyClientResult<T> = Result<T, ErrorAgencyClient>;
