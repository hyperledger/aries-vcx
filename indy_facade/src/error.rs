use std::cell::RefCell;
use std::ffi::CString;
use std::fmt;
use std::sync;

use failure::{Backtrace, Context, Fail};

use crate::utils;
use crate::utils::error;

pub mod prelude {
    pub use super::{err_msg, IndyFacadeError, VcxErrorExt, IndyFacadeErrorKind, IndyFacadeResult, VcxResultExt};
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum IndyFacadeErrorKind {
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
    #[fail(display = "Object cache error")]
    ObjectCacheError,
    #[fail(display = "Object not ready for specified action")]
    NotReady,
    #[fail(display = "IO Error, possibly creating a backup wallet")]
    IOError,
    #[fail(display = "Object (json, config, key, credential and etc...) passed to libindy has invalid structure")]
    LibindyInvalidStructure,
    #[fail(display = "Waiting for callback timed out")]
    TimeoutLibindy,
    #[fail(display = "Parameter passed to libindy was invalid")]
    InvalidLibindyParam,
    #[fail(display = "Library already initialized")]
    AlreadyInitialized,
    #[fail(display = "Action is not supported")]
    ActionNotSupported,

    // Connection
    #[fail(display = "Could not create connection")]
    CreateConnection,
    #[fail(display = "Invalid Connection Handle")]
    InvalidConnectionHandle,
    #[fail(display = "Invalid invite details structure")]
    InvalidInviteDetail,
    #[fail(display = "Invalid redirect details structure")]
    InvalidRedirectDetail,
    #[fail(display = "Cannot Delete Connection. Check status of connection is appropriate to be deleted from agency.")]
    DeleteConnection,
    #[fail(display = "Error with Connection")]
    GeneralConnectionError,

    // Payment
    #[fail(display = "No payment information associated with object")]
    NoPaymentInformation,
    #[fail(display = "Insufficient amount of tokens to process request")]
    InsufficientTokenAmount,
    #[fail(display = "Invalid payment address")]
    InvalidPaymentAddress,

    // Credential Definition error
    #[fail(display = "Call to create Credential Definition failed")]
    CreateCredDef,
    #[fail(display = "Can't create, Credential Def already on ledger")]
    CredDefAlreadyCreated,
    #[fail(display = "Invalid Credential Definition handle")]
    InvalidCredDefHandle,
    #[fail(display = "No revocation delta found in storage for this revocation registry. Were any credentials locally revoked?")]
    RevDeltaNotFound,

    // Revocation
    #[fail(display = "Failed to create Revocation Registration Definition")]
    CreateRevRegDef,
    #[fail(display = "Invalid Revocation Details")]
    InvalidRevocationDetails,
    #[fail(display = "Unable to Update Revocation Delta On Ledger")]
    InvalidRevocationEntry,
    #[fail(display = "Invalid Credential Revocation timestamp")]
    InvalidRevocationTimestamp,
    #[fail(display = "No revocation definition found")]
    RevRegDefNotFound,

    // Credential
    #[fail(display = "Invalid credential handle")]
    InvalidCredentialHandle,
    #[fail(display = "could not create credential request")]
    CreateCredentialRequest,

    // Issuer Credential
    #[fail(display = "Invalid Credential Issuer Handle")]
    InvalidIssuerCredentialHandle,
    #[fail(display = "Invalid Credential Request")]
    InvalidCredentialRequest,
    #[fail(display = "Invalid credential json")]
    InvalidCredential,
    #[fail(display = "Attributes provided to Credential Offer are not correct, possibly malformed")]
    InvalidAttributesStructure,

    // Proof
    #[fail(display = "Invalid proof handle")]
    InvalidProofHandle,
    #[fail(display = "Obj was not found with handle")]
    InvalidDisclosedProofHandle,
    #[fail(display = "Proof had invalid format")]
    InvalidProof,
    #[fail(display = "Schema was invalid or corrupt")]
    InvalidSchema,
    #[fail(display = "The Proof received does not have valid credentials listed.")]
    InvalidProofCredentialData,
    #[fail(display = "Could not create proof")]
    CreateProof,
    #[fail(display = "Proof Request Passed into Libindy Call Was Invalid")]
    InvalidProofRequest,

    // Schema
    #[fail(display = "Could not create schema")]
    CreateSchema,
    #[fail(display = "Invalid Schema Handle")]
    InvalidSchemaHandle,
    #[fail(display = "No Schema for that schema sequence number")]
    InvalidSchemaSeqNo,
    #[fail(display = "Duplicate Schema: Ledger Already Contains Schema For Given DID, Version, and Name Combination")]
    DuplicationSchema,
    #[fail(display = "Unknown Rejection of Schema Creation, refer to libindy documentation")]
    UnknownSchemaRejection,

    // Pool
    #[fail(display = "Invalid genesis transactions path.")]
    InvalidGenesisTxnPath,
    #[fail(display = "Formatting for Pool Config are incorrect.")]
    CreatePoolConfig,
    #[fail(display = "Connection to Pool Ledger.")]
    PoolLedgerConnect,
    #[fail(display = "Invalid response from ledger for paid transaction")]
    InvalidLedgerResponse,
    #[fail(display = "No Pool open. Can't return handle.")]
    NoPoolOpen,
    #[fail(display = "Message failed in post")]
    PostMessageFailed,

    // Wallet
    #[fail(display = "Error Creating a wallet")]
    WalletCreate,
    #[fail(display = "Missing wallet name in config")]
    MissingWalletName,
    #[fail(display = "Missing exported wallet path in config")]
    MissingExportedWalletPath,
    #[fail(display = "Missing exported backup key in config")]
    MissingBackupKey,
    #[fail(display = "Attempt to open wallet with invalid credentials")]
    WalletAccessFailed,
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

    // Logger
    #[fail(display = "Logging Error")]
    LoggingError,

    // Validation
    #[fail(display = "Could not encode string to a big integer.")]
    EncodeError,
    #[fail(display = "Unknown Error")]
    UnknownError,
    #[fail(display = "Invalid DID")]
    InvalidDid,
    #[fail(display = "Invalid VERKEY")]
    InvalidVerkey,
    #[fail(display = "Invalid NONCE")]
    InvalidNonce,
    #[fail(display = "Invalid URL")]
    InvalidUrl,
    #[fail(display = "Configuration is missing the Payment Method parameter")]
    MissingPaymentMethod,
    #[fail(display = "Unable to serialize")]
    SerializationError,
    #[fail(display = "Value needs to be base58")]
    NotBase58,

    // A2A
    #[fail(display = "Invalid HTTP response.")]
    InvalidHttpResponse,
    #[fail(display = "No Endpoint set for Connection Object")]
    NoEndpoint,
    #[fail(display = "Error Retrieving messages from API")]
    InvalidMessages,

    #[fail(display = "Common error {}", 0)]
    Common(u32),
    #[fail(display = "Libndy error {}", 0)]
    LibndyError(u32),
    #[fail(display = "Unknown libindy error")]
    UnknownLibndyError,
    #[fail(display = "No Agent pairwise information")]
    NoAgentInformation,

    #[fail(display = "Invalid message format")]
    InvalidMessageFormat,

    #[fail(display = "Attempted to unlock poisoned lock")]
    PoisonedLock,
}

#[derive(Debug)]
pub struct IndyFacadeError {
    inner: Context<IndyFacadeErrorKind>,
}

impl Fail for IndyFacadeError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for IndyFacadeError {
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

impl IndyFacadeError {
    pub fn from_msg<D>(kind: IndyFacadeErrorKind, msg: D) -> IndyFacadeError
        where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
        IndyFacadeError { inner: Context::new(msg).context(kind) }
    }

    pub fn kind(&self) -> IndyFacadeErrorKind {
        *self.inner.get_context()
    }

    pub fn extend<D>(self, msg: D) -> IndyFacadeError
        where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
        let kind = self.kind();
        IndyFacadeError { inner: self.inner.map(|_| msg).context(kind) }
    }

    pub fn map<D>(self, kind: IndyFacadeErrorKind, msg: D) -> IndyFacadeError
        where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
        IndyFacadeError { inner: self.inner.map(|_| msg).context(kind) }
    }
}

pub fn err_msg<D>(kind: IndyFacadeErrorKind, msg: D) -> IndyFacadeError
    where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
    IndyFacadeError::from_msg(kind, msg)
}

impl From<IndyFacadeErrorKind> for IndyFacadeError {
    fn from(kind: IndyFacadeErrorKind) -> IndyFacadeError {
        IndyFacadeError::from_msg(kind, crate::utils::error::error_message(&kind.clone().into()))
    }
}

impl<T> From<sync::PoisonError<T>> for IndyFacadeError {
    fn from(_: sync::PoisonError<T>) -> Self {
        IndyFacadeError { inner: Context::new(Backtrace::new()).context(IndyFacadeErrorKind::PoisonedLock) }
    }
}

impl From<Context<IndyFacadeErrorKind>> for IndyFacadeError {
    fn from(inner: Context<IndyFacadeErrorKind>) -> IndyFacadeError {
        IndyFacadeError { inner }
    }
}

pub type IndyFacadeResult<T> = Result<T, IndyFacadeError>;

/// Extension methods for `Result`.
pub trait VcxResultExt<T, E> {
    fn to_vcx<D>(self, kind: IndyFacadeErrorKind, msg: D) -> IndyFacadeResult<T> where D: fmt::Display + Send + Sync + 'static;
}

impl<T, E> VcxResultExt<T, E> for Result<T, E> where E: Fail
{
    fn to_vcx<D>(self, kind: IndyFacadeErrorKind, msg: D) -> IndyFacadeResult<T> where D: fmt::Display + Send + Sync + 'static {
        self.map_err(|err| err.context(msg).context(kind).into())
    }
}

/// Extension methods for `Error`.
pub trait VcxErrorExt {
    fn to_indy_facade_err<D>(self, kind: IndyFacadeErrorKind, msg: D) -> IndyFacadeError where D: fmt::Display + Send + Sync + 'static;
}

impl<E> VcxErrorExt for E where E: Fail
{
    fn to_indy_facade_err<D>(self, kind: IndyFacadeErrorKind, msg: D) -> IndyFacadeError where D: fmt::Display + Send + Sync + 'static {
        self.context(format!("\n{}: {}", std::any::type_name::<E>(), msg)).context(kind).into()
    }
}

impl From<IndyFacadeError> for u32 {
    fn from(code: IndyFacadeError) -> u32 {
        code.kind().into()
    }
}

fn string_to_cstring(s: String) -> CString {
    CString::new(s).unwrap()
}

impl From<IndyFacadeErrorKind> for u32 {
    fn from(code: IndyFacadeErrorKind) -> u32 {
        match code {
            IndyFacadeErrorKind::InvalidState => error::INVALID_STATE.code_num,
            IndyFacadeErrorKind::InvalidConfiguration => error::INVALID_CONFIGURATION.code_num,
            IndyFacadeErrorKind::InvalidHandle => error::INVALID_OBJ_HANDLE.code_num,
            IndyFacadeErrorKind::InvalidJson => error::INVALID_JSON.code_num,
            IndyFacadeErrorKind::InvalidOption => error::INVALID_OPTION.code_num,
            IndyFacadeErrorKind::InvalidMessagePack => error::INVALID_MSGPACK.code_num,
            IndyFacadeErrorKind::ObjectCacheError => error::OBJECT_CACHE_ERROR.code_num,
            IndyFacadeErrorKind::NoPaymentInformation => error::NO_PAYMENT_INFORMATION.code_num,
            IndyFacadeErrorKind::NotReady => error::NOT_READY.code_num,
            IndyFacadeErrorKind::InvalidRevocationDetails => error::INVALID_REVOCATION_DETAILS.code_num,
            IndyFacadeErrorKind::GeneralConnectionError => error::CONNECTION_ERROR.code_num,
            IndyFacadeErrorKind::IOError => error::IOERROR.code_num,
            IndyFacadeErrorKind::LibindyInvalidStructure => error::LIBINDY_INVALID_STRUCTURE.code_num,
            IndyFacadeErrorKind::TimeoutLibindy => error::TIMEOUT_LIBINDY_ERROR.code_num,
            IndyFacadeErrorKind::InvalidLibindyParam => error::INVALID_LIBINDY_PARAM.code_num,
            IndyFacadeErrorKind::AlreadyInitialized => error::ALREADY_INITIALIZED.code_num,
            IndyFacadeErrorKind::CreateConnection => error::CREATE_CONNECTION_ERROR.code_num,
            IndyFacadeErrorKind::InvalidConnectionHandle => error::INVALID_CONNECTION_HANDLE.code_num,
            IndyFacadeErrorKind::InvalidInviteDetail => error::INVALID_INVITE_DETAILS.code_num,
            IndyFacadeErrorKind::InvalidRedirectDetail => error::INVALID_REDIRECT_DETAILS.code_num,
            IndyFacadeErrorKind::DeleteConnection => error::CANNOT_DELETE_CONNECTION.code_num,
            IndyFacadeErrorKind::CreateCredDef => error::CREATE_CREDENTIAL_DEF_ERR.code_num,
            IndyFacadeErrorKind::CredDefAlreadyCreated => error::CREDENTIAL_DEF_ALREADY_CREATED.code_num,
            IndyFacadeErrorKind::InvalidCredDefHandle => error::INVALID_CREDENTIAL_DEF_HANDLE.code_num,
            IndyFacadeErrorKind::InvalidRevocationEntry => error::INVALID_REV_ENTRY.code_num,
            IndyFacadeErrorKind::CreateRevRegDef => error::INVALID_REV_REG_DEF_CREATION.code_num,
            IndyFacadeErrorKind::InvalidCredentialHandle => error::INVALID_CREDENTIAL_HANDLE.code_num,
            IndyFacadeErrorKind::CreateCredentialRequest => error::CREATE_CREDENTIAL_REQUEST_ERROR.code_num,
            IndyFacadeErrorKind::InvalidIssuerCredentialHandle => error::INVALID_ISSUER_CREDENTIAL_HANDLE.code_num,
            IndyFacadeErrorKind::InvalidCredentialRequest => error::INVALID_CREDENTIAL_REQUEST.code_num,
            IndyFacadeErrorKind::InvalidCredential => error::INVALID_CREDENTIAL_JSON.code_num,
            IndyFacadeErrorKind::InsufficientTokenAmount => error::INSUFFICIENT_TOKEN_AMOUNT.code_num,
            IndyFacadeErrorKind::InvalidProofHandle => error::INVALID_PROOF_HANDLE.code_num,
            IndyFacadeErrorKind::InvalidDisclosedProofHandle => error::INVALID_DISCLOSED_PROOF_HANDLE.code_num,
            IndyFacadeErrorKind::InvalidProof => error::INVALID_PROOF.code_num,
            IndyFacadeErrorKind::InvalidSchema => error::INVALID_SCHEMA.code_num,
            IndyFacadeErrorKind::InvalidProofCredentialData => error::INVALID_PROOF_CREDENTIAL_DATA.code_num,
            IndyFacadeErrorKind::CreateProof => error::CREATE_PROOF_ERROR.code_num,
            IndyFacadeErrorKind::InvalidRevocationTimestamp => error::INVALID_REVOCATION_TIMESTAMP.code_num,
            IndyFacadeErrorKind::CreateSchema => error::INVALID_SCHEMA_CREATION.code_num,
            IndyFacadeErrorKind::InvalidSchemaHandle => error::INVALID_SCHEMA_HANDLE.code_num,
            IndyFacadeErrorKind::InvalidSchemaSeqNo => error::INVALID_SCHEMA_SEQ_NO.code_num,
            IndyFacadeErrorKind::DuplicationSchema => error::DUPLICATE_SCHEMA.code_num,
            IndyFacadeErrorKind::UnknownSchemaRejection => error::UNKNOWN_SCHEMA_REJECTION.code_num,
            IndyFacadeErrorKind::WalletCreate => error::INVALID_WALLET_CREATION.code_num,
            IndyFacadeErrorKind::MissingWalletName => error::MISSING_WALLET_NAME.code_num,
            IndyFacadeErrorKind::WalletAccessFailed => error::WALLET_ACCESS_FAILED.code_num,
            IndyFacadeErrorKind::InvalidWalletHandle => error::INVALID_WALLET_HANDLE.code_num,
            IndyFacadeErrorKind::DuplicationWallet => error::WALLET_ALREADY_EXISTS.code_num,
            IndyFacadeErrorKind::WalletNotFound => error::WALLET_NOT_FOUND.code_num,
            IndyFacadeErrorKind::WalletRecordNotFound => error::WALLET_RECORD_NOT_FOUND.code_num,
            IndyFacadeErrorKind::PoolLedgerConnect => error::POOL_LEDGER_CONNECT.code_num,
            IndyFacadeErrorKind::InvalidGenesisTxnPath => error::INVALID_GENESIS_TXN_PATH.code_num,
            IndyFacadeErrorKind::CreatePoolConfig => error::CREATE_POOL_CONFIG.code_num,
            IndyFacadeErrorKind::DuplicationWalletRecord => error::DUPLICATE_WALLET_RECORD.code_num,
            IndyFacadeErrorKind::WalletAlreadyOpen => error::WALLET_ALREADY_OPEN.code_num,
            IndyFacadeErrorKind::DuplicationMasterSecret => error::DUPLICATE_MASTER_SECRET.code_num,
            IndyFacadeErrorKind::DuplicationDid => error::DID_ALREADY_EXISTS_IN_WALLET.code_num,
            IndyFacadeErrorKind::InvalidLedgerResponse => error::INVALID_LEDGER_RESPONSE.code_num,
            IndyFacadeErrorKind::InvalidAttributesStructure => error::INVALID_ATTRIBUTES_STRUCTURE.code_num,
            IndyFacadeErrorKind::InvalidPaymentAddress => error::INVALID_PAYMENT_ADDRESS.code_num,
            IndyFacadeErrorKind::NoEndpoint => error::NO_ENDPOINT.code_num,
            IndyFacadeErrorKind::InvalidProofRequest => error::INVALID_PROOF_REQUEST.code_num,
            IndyFacadeErrorKind::NoPoolOpen => error::NO_POOL_OPEN.code_num,
            IndyFacadeErrorKind::PostMessageFailed => error::POST_MSG_FAILURE.code_num,
            IndyFacadeErrorKind::LoggingError => error::LOGGING_ERROR.code_num,
            IndyFacadeErrorKind::EncodeError => error::BIG_NUMBER_ERROR.code_num,
            IndyFacadeErrorKind::UnknownError => error::UNKNOWN_ERROR.code_num,
            IndyFacadeErrorKind::InvalidDid => error::INVALID_DID.code_num,
            IndyFacadeErrorKind::InvalidVerkey => error::INVALID_VERKEY.code_num,
            IndyFacadeErrorKind::InvalidNonce => error::INVALID_NONCE.code_num,
            IndyFacadeErrorKind::InvalidUrl => error::INVALID_URL.code_num,
            IndyFacadeErrorKind::MissingWalletKey => error::MISSING_WALLET_KEY.code_num,
            IndyFacadeErrorKind::MissingPaymentMethod => error::MISSING_PAYMENT_METHOD.code_num,
            IndyFacadeErrorKind::SerializationError => error::SERIALIZATION_ERROR.code_num,
            IndyFacadeErrorKind::NotBase58 => error::NOT_BASE58.code_num,
            IndyFacadeErrorKind::InvalidHttpResponse => error::INVALID_HTTP_RESPONSE.code_num,
            IndyFacadeErrorKind::InvalidMessages => error::INVALID_MESSAGES.code_num,
            IndyFacadeErrorKind::MissingExportedWalletPath => error::MISSING_EXPORTED_WALLET_PATH.code_num,
            IndyFacadeErrorKind::MissingBackupKey => error::MISSING_BACKUP_KEY.code_num,
            IndyFacadeErrorKind::UnknownLibndyError => error::UNKNOWN_LIBINDY_ERROR.code_num,
            IndyFacadeErrorKind::ActionNotSupported => error::ACTION_NOT_SUPPORTED.code_num,
            IndyFacadeErrorKind::Common(num) => num,
            IndyFacadeErrorKind::LibndyError(num) => num,
            IndyFacadeErrorKind::NoAgentInformation => error::NO_AGENT_INFO.code_num,
            IndyFacadeErrorKind::RevRegDefNotFound => error::REV_REG_DEF_NOT_FOUND.code_num,
            IndyFacadeErrorKind::RevDeltaNotFound => error::REV_DELTA_NOT_FOUND.code_num,
            IndyFacadeErrorKind::PoisonedLock => error::POISONED_LOCK.code_num,
            IndyFacadeErrorKind::InvalidMessageFormat => error::INVALID_MESSAGE_FORMAT.code_num
        }
    }
}

impl From<u32> for IndyFacadeErrorKind {
    fn from(code: u32) -> IndyFacadeErrorKind {
        match code {
            _ if { error::INVALID_STATE.code_num == code } => IndyFacadeErrorKind::InvalidState,
            _ if { error::INVALID_CONFIGURATION.code_num == code } => IndyFacadeErrorKind::InvalidConfiguration,
            _ if { error::INVALID_OBJ_HANDLE.code_num == code } => IndyFacadeErrorKind::InvalidHandle,
            _ if { error::INVALID_JSON.code_num == code } => IndyFacadeErrorKind::InvalidJson,
            _ if { error::INVALID_OPTION.code_num == code } => IndyFacadeErrorKind::InvalidOption,
            _ if { error::INVALID_MSGPACK.code_num == code } => IndyFacadeErrorKind::InvalidMessagePack,
            _ if { error::OBJECT_CACHE_ERROR.code_num == code } => IndyFacadeErrorKind::ObjectCacheError,
            _ if { error::NO_PAYMENT_INFORMATION.code_num == code } => IndyFacadeErrorKind::NoPaymentInformation,
            _ if { error::NOT_READY.code_num == code } => IndyFacadeErrorKind::NotReady,
            _ if { error::INVALID_REVOCATION_DETAILS.code_num == code } => IndyFacadeErrorKind::InvalidRevocationDetails,
            _ if { error::CONNECTION_ERROR.code_num == code } => IndyFacadeErrorKind::GeneralConnectionError,
            _ if { error::IOERROR.code_num == code } => IndyFacadeErrorKind::IOError,
            _ if { error::LIBINDY_INVALID_STRUCTURE.code_num == code } => IndyFacadeErrorKind::LibindyInvalidStructure,
            _ if { error::TIMEOUT_LIBINDY_ERROR.code_num == code } => IndyFacadeErrorKind::TimeoutLibindy,
            _ if { error::INVALID_LIBINDY_PARAM.code_num == code } => IndyFacadeErrorKind::InvalidLibindyParam,
            _ if { error::ALREADY_INITIALIZED.code_num == code } => IndyFacadeErrorKind::AlreadyInitialized,
            _ if { error::CREATE_CONNECTION_ERROR.code_num == code } => IndyFacadeErrorKind::CreateConnection,
            _ if { error::INVALID_CONNECTION_HANDLE.code_num == code } => IndyFacadeErrorKind::InvalidConnectionHandle,
            _ if { error::INVALID_INVITE_DETAILS.code_num == code } => IndyFacadeErrorKind::InvalidInviteDetail,
            _ if { error::INVALID_REDIRECT_DETAILS.code_num == code } => IndyFacadeErrorKind::InvalidRedirectDetail,
            _ if { error::CANNOT_DELETE_CONNECTION.code_num == code } => IndyFacadeErrorKind::DeleteConnection,
            _ if { error::CREATE_CREDENTIAL_DEF_ERR.code_num == code } => IndyFacadeErrorKind::CreateCredDef,
            _ if { error::CREDENTIAL_DEF_ALREADY_CREATED.code_num == code } => IndyFacadeErrorKind::CredDefAlreadyCreated,
            _ if { error::INVALID_CREDENTIAL_DEF_HANDLE.code_num == code } => IndyFacadeErrorKind::InvalidCredDefHandle,
            _ if { error::INVALID_REV_ENTRY.code_num == code } => IndyFacadeErrorKind::InvalidRevocationEntry,
            _ if { error::INVALID_REV_REG_DEF_CREATION.code_num == code } => IndyFacadeErrorKind::CreateRevRegDef,
            _ if { error::INVALID_CREDENTIAL_HANDLE.code_num == code } => IndyFacadeErrorKind::InvalidCredentialHandle,
            _ if { error::CREATE_CREDENTIAL_REQUEST_ERROR.code_num == code } => IndyFacadeErrorKind::CreateCredentialRequest,
            _ if { error::INVALID_ISSUER_CREDENTIAL_HANDLE.code_num == code } => IndyFacadeErrorKind::InvalidIssuerCredentialHandle,
            _ if { error::INVALID_CREDENTIAL_REQUEST.code_num == code } => IndyFacadeErrorKind::InvalidCredentialRequest,
            _ if { error::INVALID_CREDENTIAL_JSON.code_num == code } => IndyFacadeErrorKind::InvalidCredential,
            _ if { error::INSUFFICIENT_TOKEN_AMOUNT.code_num == code } => IndyFacadeErrorKind::InsufficientTokenAmount,
            _ if { error::INVALID_PROOF_HANDLE.code_num == code } => IndyFacadeErrorKind::InvalidProofHandle,
            _ if { error::INVALID_DISCLOSED_PROOF_HANDLE.code_num == code } => IndyFacadeErrorKind::InvalidDisclosedProofHandle,
            _ if { error::INVALID_PROOF.code_num == code } => IndyFacadeErrorKind::InvalidProof,
            _ if { error::INVALID_SCHEMA.code_num == code } => IndyFacadeErrorKind::InvalidSchema,
            _ if { error::INVALID_PROOF_CREDENTIAL_DATA.code_num == code } => IndyFacadeErrorKind::InvalidProofCredentialData,
            _ if { error::CREATE_PROOF_ERROR.code_num == code } => IndyFacadeErrorKind::CreateProof,
            _ if { error::INVALID_REVOCATION_TIMESTAMP.code_num == code } => IndyFacadeErrorKind::InvalidRevocationTimestamp,
            _ if { error::INVALID_SCHEMA_CREATION.code_num == code } => IndyFacadeErrorKind::CreateSchema,
            _ if { error::INVALID_SCHEMA_HANDLE.code_num == code } => IndyFacadeErrorKind::InvalidSchemaHandle,
            _ if { error::INVALID_SCHEMA_SEQ_NO.code_num == code } => IndyFacadeErrorKind::InvalidSchemaSeqNo,
            _ if { error::DUPLICATE_SCHEMA.code_num == code } => IndyFacadeErrorKind::DuplicationSchema,
            _ if { error::UNKNOWN_SCHEMA_REJECTION.code_num == code } => IndyFacadeErrorKind::UnknownSchemaRejection,
            _ if { error::INVALID_WALLET_CREATION.code_num == code } => IndyFacadeErrorKind::WalletCreate,
            _ if { error::MISSING_WALLET_NAME.code_num == code } => IndyFacadeErrorKind::MissingWalletName,
            _ if { error::WALLET_ACCESS_FAILED.code_num == code } => IndyFacadeErrorKind::WalletAccessFailed,
            _ if { error::INVALID_WALLET_HANDLE.code_num == code } => IndyFacadeErrorKind::InvalidWalletHandle,
            _ if { error::WALLET_ALREADY_EXISTS.code_num == code } => IndyFacadeErrorKind::DuplicationWallet,
            _ if { error::WALLET_NOT_FOUND.code_num == code } => IndyFacadeErrorKind::WalletNotFound,
            _ if { error::WALLET_RECORD_NOT_FOUND.code_num == code } => IndyFacadeErrorKind::WalletRecordNotFound,
            _ if { error::POOL_LEDGER_CONNECT.code_num == code } => IndyFacadeErrorKind::PoolLedgerConnect,
            _ if { error::INVALID_GENESIS_TXN_PATH.code_num == code } => IndyFacadeErrorKind::InvalidGenesisTxnPath,
            _ if { error::CREATE_POOL_CONFIG.code_num == code } => IndyFacadeErrorKind::CreatePoolConfig,
            _ if { error::DUPLICATE_WALLET_RECORD.code_num == code } => IndyFacadeErrorKind::DuplicationWalletRecord,
            _ if { error::WALLET_ALREADY_OPEN.code_num == code } => IndyFacadeErrorKind::WalletAlreadyOpen,
            _ if { error::DUPLICATE_MASTER_SECRET.code_num == code } => IndyFacadeErrorKind::DuplicationMasterSecret,
            _ if { error::DID_ALREADY_EXISTS_IN_WALLET.code_num == code } => IndyFacadeErrorKind::DuplicationDid,
            _ if { error::INVALID_LEDGER_RESPONSE.code_num == code } => IndyFacadeErrorKind::InvalidLedgerResponse,
            _ if { error::INVALID_ATTRIBUTES_STRUCTURE.code_num == code } => IndyFacadeErrorKind::InvalidAttributesStructure,
            _ if { error::INVALID_PAYMENT_ADDRESS.code_num == code } => IndyFacadeErrorKind::InvalidPaymentAddress,
            _ if { error::NO_ENDPOINT.code_num == code } => IndyFacadeErrorKind::NoEndpoint,
            _ if { error::INVALID_PROOF_REQUEST.code_num == code } => IndyFacadeErrorKind::InvalidProofRequest,
            _ if { error::NO_POOL_OPEN.code_num == code } => IndyFacadeErrorKind::NoPoolOpen,
            _ if { error::POST_MSG_FAILURE.code_num == code } => IndyFacadeErrorKind::PostMessageFailed,
            _ if { error::LOGGING_ERROR.code_num == code } => IndyFacadeErrorKind::LoggingError,
            _ if { error::BIG_NUMBER_ERROR.code_num == code } => IndyFacadeErrorKind::EncodeError,
            _ if { error::UNKNOWN_ERROR.code_num == code } => IndyFacadeErrorKind::UnknownError,
            _ if { error::INVALID_DID.code_num == code } => IndyFacadeErrorKind::InvalidDid,
            _ if { error::INVALID_VERKEY.code_num == code } => IndyFacadeErrorKind::InvalidVerkey,
            _ if { error::INVALID_NONCE.code_num == code } => IndyFacadeErrorKind::InvalidNonce,
            _ if { error::INVALID_URL.code_num == code } => IndyFacadeErrorKind::InvalidUrl,
            _ if { error::MISSING_WALLET_KEY.code_num == code } => IndyFacadeErrorKind::MissingWalletKey,
            _ if { error::MISSING_PAYMENT_METHOD.code_num == code } => IndyFacadeErrorKind::MissingPaymentMethod,
            _ if { error::SERIALIZATION_ERROR.code_num == code } => IndyFacadeErrorKind::SerializationError,
            _ if { error::NOT_BASE58.code_num == code } => IndyFacadeErrorKind::NotBase58,
            _ if { error::INVALID_HTTP_RESPONSE.code_num == code } => IndyFacadeErrorKind::InvalidHttpResponse,
            _ if { error::INVALID_MESSAGES.code_num == code } => IndyFacadeErrorKind::InvalidMessages,
            _ if { error::MISSING_EXPORTED_WALLET_PATH.code_num == code } => IndyFacadeErrorKind::MissingExportedWalletPath,
            _ if { error::MISSING_BACKUP_KEY.code_num == code } => IndyFacadeErrorKind::MissingBackupKey,
            _ if { error::UNKNOWN_LIBINDY_ERROR.code_num == code } => IndyFacadeErrorKind::UnknownLibndyError,
            _ if { error::ACTION_NOT_SUPPORTED.code_num == code } => IndyFacadeErrorKind::ActionNotSupported,
            _ if { error::NO_AGENT_INFO.code_num == code } => IndyFacadeErrorKind::NoAgentInformation,
            _ if { error::REV_REG_DEF_NOT_FOUND.code_num == code } => IndyFacadeErrorKind::RevRegDefNotFound,
            _ if { error::REV_DELTA_NOT_FOUND.code_num == code } => IndyFacadeErrorKind::RevDeltaNotFound,
            _ => IndyFacadeErrorKind::UnknownError,
        }
    }
}
