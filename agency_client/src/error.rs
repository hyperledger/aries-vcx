use std::fmt;
use std::error::Error;

use thiserror;

use crate::utils::error_utils::kind_to_error_message;

pub mod prelude {
    pub use super::*;
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
    #[error("Object (json, config, key, credential and etc...) passed to libindy has invalid structure")]
    LibindyInvalidStructure,
    #[error("Waiting for callback timed out")]
    TimeoutLibindy,
    #[error("Parameter passed to libindy was invalid")]
    InvalidLibindyParam,

    #[error("Message failed in post")]
    PostMessageFailed,

    // Wallet
    #[error("Invalid Wallet or Search Handle")]
    InvalidWalletHandle,
    #[error("Indy wallet already exists")]
    DuplicationWallet,
    #[error("Wallet record not found")]
    WalletRecordNotFound,
    #[error("Record already exists in the wallet")]
    DuplicationWalletRecord,
    #[error("Wallet not found")]
    WalletNotFound,
    #[error("Indy wallet already open")]
    WalletAlreadyOpen,
    #[error("Configuration is missing wallet key")]
    MissingWalletKey,
    #[error("Attempted to add a Master Secret that already existed in wallet")]
    DuplicationMasterSecret,
    #[error("Attempted to add a DID to wallet when that DID already exists in wallet")]
    DuplicationDid,

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

    #[error("Failed to create agency client")]
    CreateAgent,

    #[error("Libndy error {}", 0)]
    LibndyError(u32),
    #[error("Unknown libindy error")]
    UnknownLibndyError,
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

    pub fn find_root_cause(&self) -> String {
        let mut current = self.source();
        while let Some(cause) = current {
            if cause.source().is_none() { return cause.to_string() }
            current = cause.source();
        }
        self.to_string()
    }


    pub fn kind(&self) -> AgencyClientErrorKind {
        self.kind
    }

    pub fn extend<D>(self, msg: D) -> AgencyClientError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        AgencyClientError {
            msg: msg.to_string(),
            ..self
        }
    }

    pub fn map<D>(self, kind: AgencyClientErrorKind, msg: D) -> AgencyClientError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        AgencyClientError {
            msg: msg.to_string(),
            kind,
            ..self
        }
    }
}

impl From<AgencyClientErrorKind> for AgencyClientError {
    fn from(kind: AgencyClientErrorKind) -> AgencyClientError {
        AgencyClientError::from_msg(kind, kind_to_error_message(&kind))
    }
}

impl From<serde_json::Error> for AgencyClientError {
    fn from(_err: serde_json::Error) -> Self {
        AgencyClientErrorKind::InvalidJson.into()
    }
}

pub type AgencyClientResult<T> = Result<T, AgencyClientError>;
