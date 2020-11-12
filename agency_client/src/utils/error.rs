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
pub enum AgencyCommErrorKind {
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

    #[fail(display = "Common error {}", 0)]
    Common(u32),
    #[fail(display = "Libndy error {}", 0)]
    LibndyError(u32),
    #[fail(display = "Unknown libindy error")]
    UnknownLibndyError,
}

#[derive(Debug)]
pub struct AgencyCommError {
    inner: Context<AgencyCommErrorKind>
}

impl Fail for AgencyCommError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for AgencyCommError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;

        for cause in Fail::iter_chain(&self.inner) {
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

impl AgencyCommError {
    pub fn from_msg<D>(kind: AgencyCommErrorKind, msg: D) -> AgencyCommError
        where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
        AgencyCommError { inner: Context::new(msg).context(kind) }
    }

    pub fn kind(&self) -> AgencyCommErrorKind {
        *self.inner.get_context()
    }

    pub fn extend<D>(self, msg: D) -> AgencyCommError
        where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
        let kind = self.kind();
        AgencyCommError { inner: self.inner.map(|_| msg).context(kind) }
    }

    pub fn map<D>(self, kind: AgencyCommErrorKind, msg: D) -> AgencyCommError
        where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
        AgencyCommError { inner: self.inner.map(|_| msg).context(kind) }
    }
}

impl From<AgencyCommErrorKind> for AgencyCommError {
    fn from(kind: AgencyCommErrorKind) -> AgencyCommError {
        AgencyCommError::from_msg(kind, error_utils::error_message(&kind.clone().into()))
    }
}

impl From<Context<AgencyCommErrorKind>> for AgencyCommError {
    fn from(inner: Context<AgencyCommErrorKind>) -> AgencyCommError {
        AgencyCommError { inner }
    }
}

impl From<AgencyCommError> for u32 {
    fn from(code: AgencyCommError) -> u32 {
        set_current_error(&code);
        code.kind().into()
    }
}

impl From<AgencyCommErrorKind> for u32 {
    fn from(code: AgencyCommErrorKind) -> u32 {
        match code {
            AgencyCommErrorKind::InvalidState => error_utils::INVALID_STATE.code_num,
            AgencyCommErrorKind::InvalidConfiguration => error_utils::INVALID_CONFIGURATION.code_num,
            AgencyCommErrorKind::InvalidHandle => error_utils::INVALID_OBJ_HANDLE.code_num,
            AgencyCommErrorKind::InvalidJson => error_utils::INVALID_JSON.code_num,
            AgencyCommErrorKind::InvalidOption => error_utils::INVALID_OPTION.code_num,
            AgencyCommErrorKind::InvalidMessagePack => error_utils::INVALID_MSGPACK.code_num,
            AgencyCommErrorKind::IOError => error_utils::IOERROR.code_num,
            AgencyCommErrorKind::LibindyInvalidStructure => error_utils::LIBINDY_INVALID_STRUCTURE.code_num,
            AgencyCommErrorKind::InsufficientTokenAmount => error_utils::INSUFFICIENT_TOKEN_AMOUNT.code_num,
            AgencyCommErrorKind::CredDefAlreadyCreated => error_utils::CREDENTIAL_DEF_ALREADY_CREATED.code_num,
            AgencyCommErrorKind::TimeoutLibindy => error_utils::TIMEOUT_LIBINDY_ERROR.code_num,
            AgencyCommErrorKind::InvalidLibindyParam => error_utils::INVALID_LIBINDY_PARAM.code_num,
            AgencyCommErrorKind::InvalidWalletHandle => error_utils::INVALID_WALLET_HANDLE.code_num,
            AgencyCommErrorKind::DuplicationWallet => error_utils::WALLET_ALREADY_EXISTS.code_num,
            AgencyCommErrorKind::WalletNotFound => error_utils::WALLET_NOT_FOUND.code_num,
            AgencyCommErrorKind::WalletRecordNotFound => error_utils::WALLET_RECORD_NOT_FOUND.code_num,
            AgencyCommErrorKind::CreatePoolConfig => error_utils::CREATE_POOL_CONFIG.code_num,
            AgencyCommErrorKind::DuplicationWalletRecord => error_utils::DUPLICATE_WALLET_RECORD.code_num,
            AgencyCommErrorKind::WalletAlreadyOpen => error_utils::WALLET_ALREADY_OPEN.code_num,
            AgencyCommErrorKind::DuplicationMasterSecret => error_utils::DUPLICATE_MASTER_SECRET.code_num,
            AgencyCommErrorKind::DuplicationDid => error_utils::DID_ALREADY_EXISTS_IN_WALLET.code_num,
            AgencyCommErrorKind::PostMessageFailed => error_utils::POST_MSG_FAILURE.code_num,
            AgencyCommErrorKind::UnknownError => error_utils::UNKNOWN_ERROR.code_num,
            AgencyCommErrorKind::InvalidDid => error_utils::INVALID_DID.code_num,
            AgencyCommErrorKind::InvalidVerkey => error_utils::INVALID_VERKEY.code_num,
            AgencyCommErrorKind::InvalidUrl => error_utils::INVALID_URL.code_num,
            AgencyCommErrorKind::MissingWalletKey => error_utils::MISSING_WALLET_KEY.code_num,
            AgencyCommErrorKind::SerializationError => error_utils::SERIALIZATION_ERROR.code_num,
            AgencyCommErrorKind::NotBase58 => error_utils::NOT_BASE58.code_num,
            AgencyCommErrorKind::InvalidHttpResponse => error_utils::INVALID_HTTP_RESPONSE.code_num,
            AgencyCommErrorKind::UnknownLibndyError => error_utils::UNKNOWN_LIBINDY_ERROR.code_num,
            AgencyCommErrorKind::Common(num) => num,
            AgencyCommErrorKind::LibndyError(num) => num,
        }
    }
}

impl From<u32> for AgencyCommErrorKind {
    fn from(code: u32) -> AgencyCommErrorKind {
        match code {
            _ if { error_utils::INVALID_STATE.code_num == code } => AgencyCommErrorKind::InvalidState,
            _ if { error_utils::INVALID_CONFIGURATION.code_num == code } => AgencyCommErrorKind::InvalidConfiguration,
            _ if { error_utils::INVALID_OBJ_HANDLE.code_num == code } => AgencyCommErrorKind::InvalidHandle,
            _ if { error_utils::INVALID_JSON.code_num == code } => AgencyCommErrorKind::InvalidJson,
            _ if { error_utils::INVALID_OPTION.code_num == code } => AgencyCommErrorKind::InvalidOption,
            _ if { error_utils::INVALID_MSGPACK.code_num == code } => AgencyCommErrorKind::InvalidMessagePack,
            _ if { error_utils::IOERROR.code_num == code } => AgencyCommErrorKind::IOError,
            _ if { error_utils::LIBINDY_INVALID_STRUCTURE.code_num == code } => AgencyCommErrorKind::LibindyInvalidStructure,
            _ if { error_utils::TIMEOUT_LIBINDY_ERROR.code_num == code } => AgencyCommErrorKind::TimeoutLibindy,
            _ if { error_utils::INVALID_LIBINDY_PARAM.code_num == code } => AgencyCommErrorKind::InvalidLibindyParam,
            _ if { error_utils::CREDENTIAL_DEF_ALREADY_CREATED.code_num == code } => AgencyCommErrorKind::CredDefAlreadyCreated,
            _ if { error_utils::INVALID_WALLET_HANDLE.code_num == code } => AgencyCommErrorKind::InvalidWalletHandle,
            _ if { error_utils::WALLET_ALREADY_EXISTS.code_num == code } => AgencyCommErrorKind::DuplicationWallet,
            _ if { error_utils::WALLET_NOT_FOUND.code_num == code } => AgencyCommErrorKind::WalletNotFound,
            _ if { error_utils::WALLET_RECORD_NOT_FOUND.code_num == code } => AgencyCommErrorKind::WalletRecordNotFound,
            _ if { error_utils::CREATE_POOL_CONFIG.code_num == code } => AgencyCommErrorKind::CreatePoolConfig,
            _ if { error_utils::DUPLICATE_WALLET_RECORD.code_num == code } => AgencyCommErrorKind::DuplicationWalletRecord,
            _ if { error_utils::WALLET_ALREADY_OPEN.code_num == code } => AgencyCommErrorKind::WalletAlreadyOpen,
            _ if { error_utils::DUPLICATE_MASTER_SECRET.code_num == code } => AgencyCommErrorKind::DuplicationMasterSecret,
            _ if { error_utils::DID_ALREADY_EXISTS_IN_WALLET.code_num == code } => AgencyCommErrorKind::DuplicationDid,
            _ if { error_utils::POST_MSG_FAILURE.code_num == code } => AgencyCommErrorKind::PostMessageFailed,
            _ if { error_utils::UNKNOWN_ERROR.code_num == code } => AgencyCommErrorKind::UnknownError,
            _ if { error_utils::INVALID_DID.code_num == code } => AgencyCommErrorKind::InvalidDid,
            _ if { error_utils::INVALID_VERKEY.code_num == code } => AgencyCommErrorKind::InvalidVerkey,
            _ if { error_utils::INVALID_URL.code_num == code } => AgencyCommErrorKind::InvalidUrl,
            _ if { error_utils::MISSING_WALLET_KEY.code_num == code } => AgencyCommErrorKind::MissingWalletKey,
            _ if { error_utils::SERIALIZATION_ERROR.code_num == code } => AgencyCommErrorKind::SerializationError,
            _ if { error_utils::NOT_BASE58.code_num == code } => AgencyCommErrorKind::NotBase58,
            _ if { error_utils::INVALID_HTTP_RESPONSE.code_num == code } => AgencyCommErrorKind::InvalidHttpResponse,
            _ if { error_utils::UNKNOWN_LIBINDY_ERROR.code_num == code } => AgencyCommErrorKind::UnknownLibndyError,
            _ => AgencyCommErrorKind::UnknownError,
        }
    }
}

impl From<IndyError> for AgencyCommError {
    fn from(error: IndyError) -> Self {
        match error.error_code as u32 {
            100..=111 => AgencyCommError::from_msg(AgencyCommErrorKind::InvalidLibindyParam, error.message),
            113 => AgencyCommError::from_msg(AgencyCommErrorKind::LibindyInvalidStructure, error.message),
            114 => AgencyCommError::from_msg(AgencyCommErrorKind::IOError, error.message),
            200 => AgencyCommError::from_msg(AgencyCommErrorKind::InvalidWalletHandle, error.message),
            203 => AgencyCommError::from_msg(AgencyCommErrorKind::DuplicationWallet, error.message),
            204 => AgencyCommError::from_msg(AgencyCommErrorKind::WalletNotFound, error.message),
            206 => AgencyCommError::from_msg(AgencyCommErrorKind::WalletAlreadyOpen, error.message),
            212 => AgencyCommError::from_msg(AgencyCommErrorKind::WalletRecordNotFound, error.message),
            213 => AgencyCommError::from_msg(AgencyCommErrorKind::DuplicationWalletRecord, error.message),
            306 => AgencyCommError::from_msg(AgencyCommErrorKind::CreatePoolConfig, error.message),
            404 => AgencyCommError::from_msg(AgencyCommErrorKind::DuplicationMasterSecret, error.message),
            407 => AgencyCommError::from_msg(AgencyCommErrorKind::CredDefAlreadyCreated, error.message),
            600 => AgencyCommError::from_msg(AgencyCommErrorKind::DuplicationDid, error.message),
            702 => AgencyCommError::from_msg(AgencyCommErrorKind::InsufficientTokenAmount, error.message),
            error_code => AgencyCommError::from_msg(AgencyCommErrorKind::LibndyError(error_code), error.message)
        }
    }
}

pub type VcxResult<T> = Result<T, AgencyCommError>;

/// Extension methods for `Result`.
pub trait VcxResultExt<T, E> {
    fn to_vcx<D>(self, kind: AgencyCommErrorKind, msg: D) -> VcxResult<T> where D: fmt::Display + Send + Sync + 'static;
}

impl<T, E> VcxResultExt<T, E> for Result<T, E> where E: Fail
{
    fn to_vcx<D>(self, kind: AgencyCommErrorKind, msg: D) -> VcxResult<T> where D: fmt::Display + Send + Sync + 'static {
        self.map_err(|err| err.context(msg).context(kind).into())
    }
}

/// Extension methods for `Error`.
pub trait VcxErrorExt {
    fn to_vcx<D>(self, kind: AgencyCommErrorKind, msg: D) -> AgencyCommError where D: fmt::Display + Send + Sync + 'static;
}

impl<E> VcxErrorExt for E where E: Fail
{
    fn to_vcx<D>(self, kind: AgencyCommErrorKind, msg: D) -> AgencyCommError where D: fmt::Display + Send + Sync + 'static {
        self.context(format!("\n{}: {}", std::any::type_name::<E>(), msg)).context(kind).into()
    }
}

thread_local! {
    pub static CURRENT_ERROR_C_JSON: RefCell<Option<CString>> = RefCell::new(None);
}

fn string_to_cstring(s: String) -> CString {
    CString::new(s).unwrap()
}

pub fn set_current_error(err: &AgencyCommError) {
    CURRENT_ERROR_C_JSON.try_with(|error| {
        let error_json = json!({
            "error": err.kind().to_string(),
            "message": err.to_string(),
            "cause": Fail::find_root_cause(err).to_string(),
            "backtrace": err.backtrace().map(|bt| bt.to_string())
        }).to_string();
        error.replace(Some(string_to_cstring(error_json)));
    })
        .map_err(|err| error!("Thread local variable access failed with: {:?}", err)).ok();
}
