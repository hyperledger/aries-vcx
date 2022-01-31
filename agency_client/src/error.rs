use std::cell::RefCell;
use std::ffi::CString;
use std::fmt;

use failure::{Backtrace, Context, Fail};
use indy::IndyError;

use crate::utils::error_utils;

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
    InvalidHandle,
    #[fail(display = "Invalid JSON string")]
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

    // Payment
    #[fail(display = "Insufficient amount of tokens to process request")]
    InsufficientTokenAmount,

    #[fail(display = "Can't create, Credential Def already on ledger")]
    CredDefAlreadyCreated,

    // Pool
    #[fail(display = "Formatting for Pool Config are incorrect.")]
    CreatePoolConfig,
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

    #[fail(display = "Common error {}", 0)]
    Common(u32),
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
        where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
        AgencyClientError { inner: Context::new(msg).context(kind) }
    }

    pub fn kind(&self) -> AgencyClientErrorKind {
        *self.inner.get_context()
    }

    pub fn extend<D>(self, msg: D) -> AgencyClientError
        where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
        let kind = self.kind();
        AgencyClientError { inner: self.inner.map(|_| msg).context(kind) }
    }

    pub fn map<D>(self, kind: AgencyClientErrorKind, msg: D) -> AgencyClientError
        where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
        AgencyClientError { inner: self.inner.map(|_| msg).context(kind) }
    }
}

impl From<AgencyClientErrorKind> for AgencyClientError {
    fn from(kind: AgencyClientErrorKind) -> AgencyClientError {
        AgencyClientError::from_msg(kind, error_utils::error_message(&kind.clone().into()))
    }
}

impl From<Context<AgencyClientErrorKind>> for AgencyClientError {
    fn from(inner: Context<AgencyClientErrorKind>) -> AgencyClientError {
        AgencyClientError { inner }
    }
}

impl From<AgencyClientError> for u32 {
    fn from(code: AgencyClientError) -> u32 {
        code.kind().into()
    }
}

impl From<AgencyClientErrorKind> for u32 {
    fn from(code: AgencyClientErrorKind) -> u32 {
        match code {
            AgencyClientErrorKind::InvalidState => error_utils::INVALID_STATE.code_num,
            AgencyClientErrorKind::InvalidConfiguration => error_utils::INVALID_CONFIGURATION.code_num,
            AgencyClientErrorKind::InvalidHandle => error_utils::INVALID_OBJ_HANDLE.code_num,
            AgencyClientErrorKind::InvalidJson => error_utils::INVALID_JSON.code_num,
            AgencyClientErrorKind::InvalidOption => error_utils::INVALID_OPTION.code_num,
            AgencyClientErrorKind::InvalidMessagePack => error_utils::INVALID_MSGPACK.code_num,
            AgencyClientErrorKind::IOError => error_utils::IOERROR.code_num,
            AgencyClientErrorKind::LibindyInvalidStructure => error_utils::LIBINDY_INVALID_STRUCTURE.code_num,
            AgencyClientErrorKind::InsufficientTokenAmount => error_utils::INSUFFICIENT_TOKEN_AMOUNT.code_num,
            AgencyClientErrorKind::CredDefAlreadyCreated => error_utils::CREDENTIAL_DEF_ALREADY_CREATED.code_num,
            AgencyClientErrorKind::TimeoutLibindy => error_utils::TIMEOUT_LIBINDY_ERROR.code_num,
            AgencyClientErrorKind::InvalidLibindyParam => error_utils::INVALID_LIBINDY_PARAM.code_num,
            AgencyClientErrorKind::InvalidWalletHandle => error_utils::INVALID_WALLET_HANDLE.code_num,
            AgencyClientErrorKind::DuplicationWallet => error_utils::WALLET_ALREADY_EXISTS.code_num,
            AgencyClientErrorKind::WalletNotFound => error_utils::WALLET_NOT_FOUND.code_num,
            AgencyClientErrorKind::WalletRecordNotFound => error_utils::WALLET_RECORD_NOT_FOUND.code_num,
            AgencyClientErrorKind::CreatePoolConfig => error_utils::CREATE_POOL_CONFIG.code_num,
            AgencyClientErrorKind::DuplicationWalletRecord => error_utils::DUPLICATE_WALLET_RECORD.code_num,
            AgencyClientErrorKind::WalletAlreadyOpen => error_utils::WALLET_ALREADY_OPEN.code_num,
            AgencyClientErrorKind::DuplicationMasterSecret => error_utils::DUPLICATE_MASTER_SECRET.code_num,
            AgencyClientErrorKind::DuplicationDid => error_utils::DID_ALREADY_EXISTS_IN_WALLET.code_num,
            AgencyClientErrorKind::PostMessageFailed => error_utils::POST_MSG_FAILURE.code_num,
            AgencyClientErrorKind::UnknownError => error_utils::UNKNOWN_ERROR.code_num,
            AgencyClientErrorKind::InvalidDid => error_utils::INVALID_DID.code_num,
            AgencyClientErrorKind::InvalidVerkey => error_utils::INVALID_VERKEY.code_num,
            AgencyClientErrorKind::InvalidUrl => error_utils::INVALID_URL.code_num,
            AgencyClientErrorKind::MissingWalletKey => error_utils::MISSING_WALLET_KEY.code_num,
            AgencyClientErrorKind::SerializationError => error_utils::SERIALIZATION_ERROR.code_num,
            AgencyClientErrorKind::NotBase58 => error_utils::NOT_BASE58.code_num,
            AgencyClientErrorKind::InvalidHttpResponse => error_utils::INVALID_HTTP_RESPONSE.code_num,
            AgencyClientErrorKind::UnknownLibndyError => error_utils::UNKNOWN_LIBINDY_ERROR.code_num,
            AgencyClientErrorKind::CreateAgent => error_utils::CREATE_AGENT.code_num,
            AgencyClientErrorKind::Common(num) => num,
            AgencyClientErrorKind::LibndyError(num) => num,
        }
    }
}

impl From<u32> for AgencyClientErrorKind {
    fn from(code: u32) -> AgencyClientErrorKind {
        match code {
            _ if { error_utils::INVALID_STATE.code_num == code } => AgencyClientErrorKind::InvalidState,
            _ if { error_utils::INVALID_CONFIGURATION.code_num == code } => AgencyClientErrorKind::InvalidConfiguration,
            _ if { error_utils::INVALID_OBJ_HANDLE.code_num == code } => AgencyClientErrorKind::InvalidHandle,
            _ if { error_utils::INVALID_JSON.code_num == code } => AgencyClientErrorKind::InvalidJson,
            _ if { error_utils::INVALID_OPTION.code_num == code } => AgencyClientErrorKind::InvalidOption,
            _ if { error_utils::INVALID_MSGPACK.code_num == code } => AgencyClientErrorKind::InvalidMessagePack,
            _ if { error_utils::IOERROR.code_num == code } => AgencyClientErrorKind::IOError,
            _ if { error_utils::LIBINDY_INVALID_STRUCTURE.code_num == code } => AgencyClientErrorKind::LibindyInvalidStructure,
            _ if { error_utils::TIMEOUT_LIBINDY_ERROR.code_num == code } => AgencyClientErrorKind::TimeoutLibindy,
            _ if { error_utils::INVALID_LIBINDY_PARAM.code_num == code } => AgencyClientErrorKind::InvalidLibindyParam,
            _ if { error_utils::CREDENTIAL_DEF_ALREADY_CREATED.code_num == code } => AgencyClientErrorKind::CredDefAlreadyCreated,
            _ if { error_utils::INVALID_WALLET_HANDLE.code_num == code } => AgencyClientErrorKind::InvalidWalletHandle,
            _ if { error_utils::WALLET_ALREADY_EXISTS.code_num == code } => AgencyClientErrorKind::DuplicationWallet,
            _ if { error_utils::WALLET_NOT_FOUND.code_num == code } => AgencyClientErrorKind::WalletNotFound,
            _ if { error_utils::WALLET_RECORD_NOT_FOUND.code_num == code } => AgencyClientErrorKind::WalletRecordNotFound,
            _ if { error_utils::CREATE_POOL_CONFIG.code_num == code } => AgencyClientErrorKind::CreatePoolConfig,
            _ if { error_utils::DUPLICATE_WALLET_RECORD.code_num == code } => AgencyClientErrorKind::DuplicationWalletRecord,
            _ if { error_utils::WALLET_ALREADY_OPEN.code_num == code } => AgencyClientErrorKind::WalletAlreadyOpen,
            _ if { error_utils::DUPLICATE_MASTER_SECRET.code_num == code } => AgencyClientErrorKind::DuplicationMasterSecret,
            _ if { error_utils::DID_ALREADY_EXISTS_IN_WALLET.code_num == code } => AgencyClientErrorKind::DuplicationDid,
            _ if { error_utils::POST_MSG_FAILURE.code_num == code } => AgencyClientErrorKind::PostMessageFailed,
            _ if { error_utils::UNKNOWN_ERROR.code_num == code } => AgencyClientErrorKind::UnknownError,
            _ if { error_utils::INVALID_DID.code_num == code } => AgencyClientErrorKind::InvalidDid,
            _ if { error_utils::INVALID_VERKEY.code_num == code } => AgencyClientErrorKind::InvalidVerkey,
            _ if { error_utils::INVALID_URL.code_num == code } => AgencyClientErrorKind::InvalidUrl,
            _ if { error_utils::MISSING_WALLET_KEY.code_num == code } => AgencyClientErrorKind::MissingWalletKey,
            _ if { error_utils::SERIALIZATION_ERROR.code_num == code } => AgencyClientErrorKind::SerializationError,
            _ if { error_utils::NOT_BASE58.code_num == code } => AgencyClientErrorKind::NotBase58,
            _ if { error_utils::INVALID_HTTP_RESPONSE.code_num == code } => AgencyClientErrorKind::InvalidHttpResponse,
            _ if { error_utils::UNKNOWN_LIBINDY_ERROR.code_num == code } => AgencyClientErrorKind::UnknownLibndyError,
            _ if { error_utils::CREATE_AGENT.code_num == code } => AgencyClientErrorKind::CreateAgent,
            _ => AgencyClientErrorKind::UnknownError,
        }
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
            306 => AgencyClientError::from_msg(AgencyClientErrorKind::CreatePoolConfig, error.message),
            404 => AgencyClientError::from_msg(AgencyClientErrorKind::DuplicationMasterSecret, error.message),
            407 => AgencyClientError::from_msg(AgencyClientErrorKind::CredDefAlreadyCreated, error.message),
            600 => AgencyClientError::from_msg(AgencyClientErrorKind::DuplicationDid, error.message),
            702 => AgencyClientError::from_msg(AgencyClientErrorKind::InsufficientTokenAmount, error.message),
            error_code => AgencyClientError::from_msg(AgencyClientErrorKind::LibndyError(error_code), error.message)
        }
    }
}

pub type AgencyClientResult<T> = Result<T, AgencyClientError>;

/// Extension methods for `Result`.
pub trait VcxResultExt<T, E> {
    fn to_vcx<D>(self, kind: AgencyClientErrorKind, msg: D) -> AgencyClientResult<T> where D: fmt::Display + Send + Sync + 'static;
}

impl<T, E> VcxResultExt<T, E> for Result<T, E> where E: Fail
{
    fn to_vcx<D>(self, kind: AgencyClientErrorKind, msg: D) -> AgencyClientResult<T> where D: fmt::Display + Send + Sync + 'static {
        self.map_err(|err| err.context(msg).context(kind).into())
    }
}

/// Extension methods for `Error`.
pub trait VcxErrorExt {
    fn to_vcx<D>(self, kind: AgencyClientErrorKind, msg: D) -> AgencyClientError where D: fmt::Display + Send + Sync + 'static;
}

impl<E> VcxErrorExt for E where E: Fail
{
    fn to_vcx<D>(self, kind: AgencyClientErrorKind, msg: D) -> AgencyClientError where D: fmt::Display + Send + Sync + 'static {
        self.context(format!("\n{}: {}", std::any::type_name::<E>(), msg)).context(kind).into()
    }
}