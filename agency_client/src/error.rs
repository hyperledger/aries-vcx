use std::fmt;

use failure::{Backtrace, Context, Fail};
use indy::IndyError;

use crate::utils::error_utils::kind_to_error_message;

pub mod prelude {
    pub use super::*;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum AgencyClientErrorKind {
    // Common
    #[fail(display = "Object is in invalid state for requested operation")]
    InvalidState,
    #[fail(display = "Invalid Configuration")]
    InvalidConfiguration,
    #[fail(display = "Obj was not found with handle")]
    InvalidJson,
    #[fail(display = "Invalid Option")]
    InvalidOption,
    #[fail(display = "Invalid MessagePack")]
    InvalidMessagePack,
    #[fail(display = "IO Error, possibly creating a backup wallet")]
    IOError,
    #[fail(display = "Object (json, config, key, credential and etc...) passed to libindy has invalid structure")]
    LibindyInvalidStructure,
    #[fail(display = "Waiting for callback timed out")]
    TimeoutLibindy,
    #[fail(display = "Parameter passed to libindy was invalid")]
    InvalidLibindyParam,

    #[fail(display = "Message failed in post")]
    PostMessageFailed,

    // Wallet
    #[fail(display = "Invalid Wallet or Search Handle")]
    InvalidWalletHandle,
    #[fail(display = "Indy wallet already exists")]
    DuplicationWallet,
    #[fail(display = "Wallet record not found")]
    WalletRecordNotFound,
    #[fail(display = "Record already exists in the wallet")]
    DuplicationWalletRecord,
    #[fail(display = "Wallet not found")]
    WalletNotFound,
    #[fail(display = "Indy wallet already open")]
    WalletAlreadyOpen,
    #[fail(display = "Configuration is missing wallet key")]
    MissingWalletKey,
    #[fail(display = "Attempted to add a Master Secret that already existed in wallet")]
    DuplicationMasterSecret,
    #[fail(display = "Attempted to add a DID to wallet when that DID already exists in wallet")]
    DuplicationDid,

    // Validation
    #[fail(display = "Unknown Error")]
    UnknownError,
    #[fail(display = "Invalid DID")]
    InvalidDid,
    #[fail(display = "Invalid VERKEY")]
    InvalidVerkey,
    #[fail(display = "Invalid URL")]
    InvalidUrl,
    #[fail(display = "Unable to serialize")]
    SerializationError,
    #[fail(display = "Value needs to be base58")]
    NotBase58,

    // A2A
    #[fail(display = "Invalid HTTP response.")]
    InvalidHttpResponse,

    #[fail(display = "Failed to create agency client")]
    CreateAgent,

    #[fail(display = "Libndy error {}", 0)]
    LibndyError(u32),
    #[fail(display = "Unknown libindy error")]
    UnknownLibndyError,
}

#[derive(Debug)]
pub struct AgencyClientError {
    inner: Context<AgencyClientErrorKind>,
}

impl Fail for AgencyClientError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for AgencyClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;

        for cause in <dyn Fail>::iter_chain(&self.inner) {
            if first {
                first = false;
                writeln!(f, "Error: {}", cause)?;
            } else {
                writeln!(f, "  Caused by: {}", cause)?;
            }
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
            inner: Context::new(msg).context(kind),
        }
    }

    pub fn kind(&self) -> AgencyClientErrorKind {
        *self.inner.get_context()
    }

    pub fn extend<D>(self, msg: D) -> AgencyClientError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        let kind = self.kind();
        AgencyClientError {
            inner: self.inner.map(|_| msg).context(kind),
        }
    }

    pub fn map<D>(self, kind: AgencyClientErrorKind, msg: D) -> AgencyClientError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        AgencyClientError {
            inner: self.inner.map(|_| msg).context(kind),
        }
    }
}

impl From<AgencyClientErrorKind> for AgencyClientError {
    fn from(kind: AgencyClientErrorKind) -> AgencyClientError {
        AgencyClientError::from_msg(kind, kind_to_error_message(&kind))
    }
}

impl From<Context<AgencyClientErrorKind>> for AgencyClientError {
    fn from(inner: Context<AgencyClientErrorKind>) -> AgencyClientError {
        AgencyClientError { inner }
    }
}

impl From<IndyError> for AgencyClientError {
    fn from(error: IndyError) -> Self {
        match error.error_code as u32 {
            100..=111 => AgencyClientError::from_msg(AgencyClientErrorKind::InvalidLibindyParam, error.message),
            113 => AgencyClientError::from_msg(AgencyClientErrorKind::LibindyInvalidStructure, error.message),
            114 => AgencyClientError::from_msg(AgencyClientErrorKind::IOError, error.message),
            200 => AgencyClientError::from_msg(AgencyClientErrorKind::InvalidWalletHandle, error.message),
            203 => AgencyClientError::from_msg(AgencyClientErrorKind::DuplicationWallet, error.message),
            204 => AgencyClientError::from_msg(AgencyClientErrorKind::WalletNotFound, error.message),
            206 => AgencyClientError::from_msg(AgencyClientErrorKind::WalletAlreadyOpen, error.message),
            212 => AgencyClientError::from_msg(AgencyClientErrorKind::WalletRecordNotFound, error.message),
            213 => AgencyClientError::from_msg(AgencyClientErrorKind::DuplicationWalletRecord, error.message),
            404 => AgencyClientError::from_msg(AgencyClientErrorKind::DuplicationMasterSecret, error.message),
            600 => AgencyClientError::from_msg(AgencyClientErrorKind::DuplicationDid, error.message),
            error_code => AgencyClientError::from_msg(AgencyClientErrorKind::LibndyError(error_code), error.message),
        }
    }
}

pub type AgencyClientResult<T> = Result<T, AgencyClientError>;
