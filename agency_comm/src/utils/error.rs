use std::cell::RefCell;
use std::ffi::CString;
use std::fmt;
use std::ptr;

use failure::{Backtrace, Context, Fail};
use libc::c_char;

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

pub fn err_msg<D>(kind: AgencyCommErrorKind, msg: D) -> AgencyCommError
    where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
    AgencyCommError::from_msg(kind, msg)
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
            AgencyCommErrorKind::ObjectCacheError => error_utils::OBJECT_CACHE_ERROR.code_num,
            AgencyCommErrorKind::NoPaymentInformation => error_utils::NO_PAYMENT_INFORMATION.code_num,
            AgencyCommErrorKind::NotReady => error_utils::NOT_READY.code_num,
            AgencyCommErrorKind::InvalidRevocationDetails => error_utils::INVALID_REVOCATION_DETAILS.code_num,
            AgencyCommErrorKind::GeneralConnectionError => error_utils::CONNECTION_ERROR.code_num,
            AgencyCommErrorKind::IOError => error_utils::IOERROR.code_num,
            AgencyCommErrorKind::LibindyInvalidStructure => error_utils::LIBINDY_INVALID_STRUCTURE.code_num,
            AgencyCommErrorKind::TimeoutLibindy => error_utils::TIMEOUT_LIBINDY_ERROR.code_num,
            AgencyCommErrorKind::InvalidLibindyParam => error_utils::INVALID_LIBINDY_PARAM.code_num,
            AgencyCommErrorKind::AlreadyInitialized => error_utils::ALREADY_INITIALIZED.code_num,
            AgencyCommErrorKind::CreateConnection => error_utils::CREATE_CONNECTION_ERROR.code_num,
            AgencyCommErrorKind::InvalidConnectionHandle => error_utils::INVALID_CONNECTION_HANDLE.code_num,
            AgencyCommErrorKind::InvalidInviteDetail => error_utils::INVALID_INVITE_DETAILS.code_num,
            AgencyCommErrorKind::InvalidRedirectDetail => error_utils::INVALID_REDIRECT_DETAILS.code_num,
            AgencyCommErrorKind::DeleteConnection => error_utils::CANNOT_DELETE_CONNECTION.code_num,
            AgencyCommErrorKind::CreateCredDef => error_utils::CREATE_CREDENTIAL_DEF_ERR.code_num,
            AgencyCommErrorKind::CredDefAlreadyCreated => error_utils::CREDENTIAL_DEF_ALREADY_CREATED.code_num,
            AgencyCommErrorKind::InvalidCredDefHandle => error_utils::INVALID_CREDENTIAL_DEF_HANDLE.code_num,
            AgencyCommErrorKind::InvalidRevocationEntry => error_utils::INVALID_REV_ENTRY.code_num,
            AgencyCommErrorKind::CreateRevRegDef => error_utils::INVALID_REV_REG_DEF_CREATION.code_num,
            AgencyCommErrorKind::InvalidCredentialHandle => error_utils::INVALID_CREDENTIAL_HANDLE.code_num,
            AgencyCommErrorKind::CreateCredentialRequest => error_utils::CREATE_CREDENTIAL_REQUEST_ERROR.code_num,
            AgencyCommErrorKind::InvalidIssuerCredentialHandle => error_utils::INVALID_ISSUER_CREDENTIAL_HANDLE.code_num,
            AgencyCommErrorKind::InvalidCredentialRequest => error_utils::INVALID_CREDENTIAL_REQUEST.code_num,
            AgencyCommErrorKind::InvalidCredential => error_utils::INVALID_CREDENTIAL_JSON.code_num,
            AgencyCommErrorKind::InsufficientTokenAmount => error_utils::INSUFFICIENT_TOKEN_AMOUNT.code_num,
            AgencyCommErrorKind::InvalidProofHandle => error_utils::INVALID_PROOF_HANDLE.code_num,
            AgencyCommErrorKind::InvalidDisclosedProofHandle => error_utils::INVALID_DISCLOSED_PROOF_HANDLE.code_num,
            AgencyCommErrorKind::InvalidProof => error_utils::INVALID_PROOF.code_num,
            AgencyCommErrorKind::InvalidSchema => error_utils::INVALID_SCHEMA.code_num,
            AgencyCommErrorKind::InvalidProofCredentialData => error_utils::INVALID_PROOF_CREDENTIAL_DATA.code_num,
            AgencyCommErrorKind::CreateProof => error_utils::CREATE_PROOF_ERROR.code_num,
            AgencyCommErrorKind::InvalidRevocationTimestamp => error_utils::INVALID_REVOCATION_TIMESTAMP.code_num,
            AgencyCommErrorKind::CreateSchema => error_utils::INVALID_SCHEMA_CREATION.code_num,
            AgencyCommErrorKind::InvalidSchemaHandle => error_utils::INVALID_SCHEMA_HANDLE.code_num,
            AgencyCommErrorKind::InvalidSchemaSeqNo => error_utils::INVALID_SCHEMA_SEQ_NO.code_num,
            AgencyCommErrorKind::DuplicationSchema => error_utils::DUPLICATE_SCHEMA.code_num,
            AgencyCommErrorKind::UnknownSchemaRejection => error_utils::UNKNOWN_SCHEMA_REJECTION.code_num,
            AgencyCommErrorKind::WalletCreate => error_utils::INVALID_WALLET_CREATION.code_num,
            AgencyCommErrorKind::MissingWalletName => error_utils::MISSING_WALLET_NAME.code_num,
            AgencyCommErrorKind::WalletAccessFailed => error_utils::WALLET_ACCESS_FAILED.code_num,
            AgencyCommErrorKind::InvalidWalletHandle => error_utils::INVALID_WALLET_HANDLE.code_num,
            AgencyCommErrorKind::DuplicationWallet => error_utils::WALLET_ALREADY_EXISTS.code_num,
            AgencyCommErrorKind::WalletNotFound => error_utils::WALLET_NOT_FOUND.code_num,
            AgencyCommErrorKind::WalletRecordNotFound => error_utils::WALLET_RECORD_NOT_FOUND.code_num,
            AgencyCommErrorKind::PoolLedgerConnect => error_utils::POOL_LEDGER_CONNECT.code_num,
            AgencyCommErrorKind::InvalidGenesisTxnPath => error_utils::INVALID_GENESIS_TXN_PATH.code_num,
            AgencyCommErrorKind::CreatePoolConfig => error_utils::CREATE_POOL_CONFIG.code_num,
            AgencyCommErrorKind::DuplicationWalletRecord => error_utils::DUPLICATE_WALLET_RECORD.code_num,
            AgencyCommErrorKind::WalletAlreadyOpen => error_utils::WALLET_ALREADY_OPEN.code_num,
            AgencyCommErrorKind::DuplicationMasterSecret => error_utils::DUPLICATE_MASTER_SECRET.code_num,
            AgencyCommErrorKind::DuplicationDid => error_utils::DID_ALREADY_EXISTS_IN_WALLET.code_num,
            AgencyCommErrorKind::InvalidLedgerResponse => error_utils::INVALID_LEDGER_RESPONSE.code_num,
            AgencyCommErrorKind::InvalidAttributesStructure => error_utils::INVALID_ATTRIBUTES_STRUCTURE.code_num,
            AgencyCommErrorKind::InvalidPaymentAddress => error_utils::INVALID_PAYMENT_ADDRESS.code_num,
            AgencyCommErrorKind::NoEndpoint => error_utils::NO_ENDPOINT.code_num,
            AgencyCommErrorKind::InvalidProofRequest => error_utils::INVALID_PROOF_REQUEST.code_num,
            AgencyCommErrorKind::NoPoolOpen => error_utils::NO_POOL_OPEN.code_num,
            AgencyCommErrorKind::PostMessageFailed => error_utils::POST_MSG_FAILURE.code_num,
            AgencyCommErrorKind::LoggingError => error_utils::LOGGING_ERROR.code_num,
            AgencyCommErrorKind::EncodeError => error_utils::BIG_NUMBER_ERROR.code_num,
            AgencyCommErrorKind::UnknownError => error_utils::UNKNOWN_ERROR.code_num,
            AgencyCommErrorKind::InvalidDid => error_utils::INVALID_DID.code_num,
            AgencyCommErrorKind::InvalidVerkey => error_utils::INVALID_VERKEY.code_num,
            AgencyCommErrorKind::InvalidNonce => error_utils::INVALID_NONCE.code_num,
            AgencyCommErrorKind::InvalidUrl => error_utils::INVALID_URL.code_num,
            AgencyCommErrorKind::MissingWalletKey => error_utils::MISSING_WALLET_KEY.code_num,
            AgencyCommErrorKind::MissingPaymentMethod => error_utils::MISSING_PAYMENT_METHOD.code_num,
            AgencyCommErrorKind::SerializationError => error_utils::SERIALIZATION_ERROR.code_num,
            AgencyCommErrorKind::NotBase58 => error_utils::NOT_BASE58.code_num,
            AgencyCommErrorKind::InvalidHttpResponse => error_utils::INVALID_HTTP_RESPONSE.code_num,
            AgencyCommErrorKind::InvalidMessages => error_utils::INVALID_MESSAGES.code_num,
            AgencyCommErrorKind::MissingExportedWalletPath => error_utils::MISSING_EXPORTED_WALLET_PATH.code_num,
            AgencyCommErrorKind::MissingBackupKey => error_utils::MISSING_BACKUP_KEY.code_num,
            AgencyCommErrorKind::UnknownLibndyError => error_utils::UNKNOWN_LIBINDY_ERROR.code_num,
            AgencyCommErrorKind::ActionNotSupported => error_utils::ACTION_NOT_SUPPORTED.code_num,
            AgencyCommErrorKind::Common(num) => num,
            AgencyCommErrorKind::LibndyError(num) => num,
            AgencyCommErrorKind::NoAgentInformation => error_utils::NO_AGENT_INFO.code_num,
            AgencyCommErrorKind::RevRegDefNotFound => error_utils::REV_REG_DEF_NOT_FOUND.code_num,
            AgencyCommErrorKind::RevDeltaNotFound => error_utils::REV_DELTA_NOT_FOUND.code_num,
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
            _ if { error_utils::OBJECT_CACHE_ERROR.code_num == code } => AgencyCommErrorKind::ObjectCacheError,
            _ if { error_utils::NO_PAYMENT_INFORMATION.code_num == code } => AgencyCommErrorKind::NoPaymentInformation,
            _ if { error_utils::NOT_READY.code_num == code } => AgencyCommErrorKind::NotReady,
            _ if { error_utils::INVALID_REVOCATION_DETAILS.code_num == code } => AgencyCommErrorKind::InvalidRevocationDetails,
            _ if { error_utils::CONNECTION_ERROR.code_num == code } => AgencyCommErrorKind::GeneralConnectionError,
            _ if { error_utils::IOERROR.code_num == code } => AgencyCommErrorKind::IOError,
            _ if { error_utils::LIBINDY_INVALID_STRUCTURE.code_num == code } => AgencyCommErrorKind::LibindyInvalidStructure,
            _ if { error_utils::TIMEOUT_LIBINDY_ERROR.code_num == code } => AgencyCommErrorKind::TimeoutLibindy,
            _ if { error_utils::INVALID_LIBINDY_PARAM.code_num == code } => AgencyCommErrorKind::InvalidLibindyParam,
            _ if { error_utils::ALREADY_INITIALIZED.code_num == code } => AgencyCommErrorKind::AlreadyInitialized,
            _ if { error_utils::CREATE_CONNECTION_ERROR.code_num == code } => AgencyCommErrorKind::CreateConnection,
            _ if { error_utils::INVALID_CONNECTION_HANDLE.code_num == code } => AgencyCommErrorKind::InvalidConnectionHandle,
            _ if { error_utils::INVALID_INVITE_DETAILS.code_num == code } => AgencyCommErrorKind::InvalidInviteDetail,
            _ if { error_utils::INVALID_REDIRECT_DETAILS.code_num == code } => AgencyCommErrorKind::InvalidRedirectDetail,
            _ if { error_utils::CANNOT_DELETE_CONNECTION.code_num == code } => AgencyCommErrorKind::DeleteConnection,
            _ if { error_utils::CREATE_CREDENTIAL_DEF_ERR.code_num == code } => AgencyCommErrorKind::CreateCredDef,
            _ if { error_utils::CREDENTIAL_DEF_ALREADY_CREATED.code_num == code } => AgencyCommErrorKind::CredDefAlreadyCreated,
            _ if { error_utils::INVALID_CREDENTIAL_DEF_HANDLE.code_num == code } => AgencyCommErrorKind::InvalidCredDefHandle,
            _ if { error_utils::INVALID_REV_ENTRY.code_num == code } => AgencyCommErrorKind::InvalidRevocationEntry,
            _ if { error_utils::INVALID_REV_REG_DEF_CREATION.code_num == code } => AgencyCommErrorKind::CreateRevRegDef,
            _ if { error_utils::INVALID_CREDENTIAL_HANDLE.code_num == code } => AgencyCommErrorKind::InvalidCredentialHandle,
            _ if { error_utils::CREATE_CREDENTIAL_REQUEST_ERROR.code_num == code } => AgencyCommErrorKind::CreateCredentialRequest,
            _ if { error_utils::INVALID_ISSUER_CREDENTIAL_HANDLE.code_num == code } => AgencyCommErrorKind::InvalidIssuerCredentialHandle,
            _ if { error_utils::INVALID_CREDENTIAL_REQUEST.code_num == code } => AgencyCommErrorKind::InvalidCredentialRequest,
            _ if { error_utils::INVALID_CREDENTIAL_JSON.code_num == code } => AgencyCommErrorKind::InvalidCredential,
            _ if { error_utils::INSUFFICIENT_TOKEN_AMOUNT.code_num == code } => AgencyCommErrorKind::InsufficientTokenAmount,
            _ if { error_utils::INVALID_PROOF_HANDLE.code_num == code } => AgencyCommErrorKind::InvalidProofHandle,
            _ if { error_utils::INVALID_DISCLOSED_PROOF_HANDLE.code_num == code } => AgencyCommErrorKind::InvalidDisclosedProofHandle,
            _ if { error_utils::INVALID_PROOF.code_num == code } => AgencyCommErrorKind::InvalidProof,
            _ if { error_utils::INVALID_SCHEMA.code_num == code } => AgencyCommErrorKind::InvalidSchema,
            _ if { error_utils::INVALID_PROOF_CREDENTIAL_DATA.code_num == code } => AgencyCommErrorKind::InvalidProofCredentialData,
            _ if { error_utils::CREATE_PROOF_ERROR.code_num == code } => AgencyCommErrorKind::CreateProof,
            _ if { error_utils::INVALID_REVOCATION_TIMESTAMP.code_num == code } => AgencyCommErrorKind::InvalidRevocationTimestamp,
            _ if { error_utils::INVALID_SCHEMA_CREATION.code_num == code } => AgencyCommErrorKind::CreateSchema,
            _ if { error_utils::INVALID_SCHEMA_HANDLE.code_num == code } => AgencyCommErrorKind::InvalidSchemaHandle,
            _ if { error_utils::INVALID_SCHEMA_SEQ_NO.code_num == code } => AgencyCommErrorKind::InvalidSchemaSeqNo,
            _ if { error_utils::DUPLICATE_SCHEMA.code_num == code } => AgencyCommErrorKind::DuplicationSchema,
            _ if { error_utils::UNKNOWN_SCHEMA_REJECTION.code_num == code } => AgencyCommErrorKind::UnknownSchemaRejection,
            _ if { error_utils::INVALID_WALLET_CREATION.code_num == code } => AgencyCommErrorKind::WalletCreate,
            _ if { error_utils::MISSING_WALLET_NAME.code_num == code } => AgencyCommErrorKind::MissingWalletName,
            _ if { error_utils::WALLET_ACCESS_FAILED.code_num == code } => AgencyCommErrorKind::WalletAccessFailed,
            _ if { error_utils::INVALID_WALLET_HANDLE.code_num == code } => AgencyCommErrorKind::InvalidWalletHandle,
            _ if { error_utils::WALLET_ALREADY_EXISTS.code_num == code } => AgencyCommErrorKind::DuplicationWallet,
            _ if { error_utils::WALLET_NOT_FOUND.code_num == code } => AgencyCommErrorKind::WalletNotFound,
            _ if { error_utils::WALLET_RECORD_NOT_FOUND.code_num == code } => AgencyCommErrorKind::WalletRecordNotFound,
            _ if { error_utils::POOL_LEDGER_CONNECT.code_num == code } => AgencyCommErrorKind::PoolLedgerConnect,
            _ if { error_utils::INVALID_GENESIS_TXN_PATH.code_num == code } => AgencyCommErrorKind::InvalidGenesisTxnPath,
            _ if { error_utils::CREATE_POOL_CONFIG.code_num == code } => AgencyCommErrorKind::CreatePoolConfig,
            _ if { error_utils::DUPLICATE_WALLET_RECORD.code_num == code } => AgencyCommErrorKind::DuplicationWalletRecord,
            _ if { error_utils::WALLET_ALREADY_OPEN.code_num == code } => AgencyCommErrorKind::WalletAlreadyOpen,
            _ if { error_utils::DUPLICATE_MASTER_SECRET.code_num == code } => AgencyCommErrorKind::DuplicationMasterSecret,
            _ if { error_utils::DID_ALREADY_EXISTS_IN_WALLET.code_num == code } => AgencyCommErrorKind::DuplicationDid,
            _ if { error_utils::INVALID_LEDGER_RESPONSE.code_num == code } => AgencyCommErrorKind::InvalidLedgerResponse,
            _ if { error_utils::INVALID_ATTRIBUTES_STRUCTURE.code_num == code } => AgencyCommErrorKind::InvalidAttributesStructure,
            _ if { error_utils::INVALID_PAYMENT_ADDRESS.code_num == code } => AgencyCommErrorKind::InvalidPaymentAddress,
            _ if { error_utils::NO_ENDPOINT.code_num == code } => AgencyCommErrorKind::NoEndpoint,
            _ if { error_utils::INVALID_PROOF_REQUEST.code_num == code } => AgencyCommErrorKind::InvalidProofRequest,
            _ if { error_utils::NO_POOL_OPEN.code_num == code } => AgencyCommErrorKind::NoPoolOpen,
            _ if { error_utils::POST_MSG_FAILURE.code_num == code } => AgencyCommErrorKind::PostMessageFailed,
            _ if { error_utils::LOGGING_ERROR.code_num == code } => AgencyCommErrorKind::LoggingError,
            _ if { error_utils::BIG_NUMBER_ERROR.code_num == code } => AgencyCommErrorKind::EncodeError,
            _ if { error_utils::UNKNOWN_ERROR.code_num == code } => AgencyCommErrorKind::UnknownError,
            _ if { error_utils::INVALID_DID.code_num == code } => AgencyCommErrorKind::InvalidDid,
            _ if { error_utils::INVALID_VERKEY.code_num == code } => AgencyCommErrorKind::InvalidVerkey,
            _ if { error_utils::INVALID_NONCE.code_num == code } => AgencyCommErrorKind::InvalidNonce,
            _ if { error_utils::INVALID_URL.code_num == code } => AgencyCommErrorKind::InvalidUrl,
            _ if { error_utils::MISSING_WALLET_KEY.code_num == code } => AgencyCommErrorKind::MissingWalletKey,
            _ if { error_utils::MISSING_PAYMENT_METHOD.code_num == code } => AgencyCommErrorKind::MissingPaymentMethod,
            _ if { error_utils::SERIALIZATION_ERROR.code_num == code } => AgencyCommErrorKind::SerializationError,
            _ if { error_utils::NOT_BASE58.code_num == code } => AgencyCommErrorKind::NotBase58,
            _ if { error_utils::INVALID_HTTP_RESPONSE.code_num == code } => AgencyCommErrorKind::InvalidHttpResponse,
            _ if { error_utils::INVALID_MESSAGES.code_num == code } => AgencyCommErrorKind::InvalidMessages,
            _ if { error_utils::MISSING_EXPORTED_WALLET_PATH.code_num == code } => AgencyCommErrorKind::MissingExportedWalletPath,
            _ if { error_utils::MISSING_BACKUP_KEY.code_num == code } => AgencyCommErrorKind::MissingBackupKey,
            _ if { error_utils::UNKNOWN_LIBINDY_ERROR.code_num == code } => AgencyCommErrorKind::UnknownLibndyError,
            _ if { error_utils::ACTION_NOT_SUPPORTED.code_num == code } => AgencyCommErrorKind::ActionNotSupported,
            _ if { error_utils::NO_AGENT_INFO.code_num == code } => AgencyCommErrorKind::NoAgentInformation,
            _ if { error_utils::REV_REG_DEF_NOT_FOUND.code_num == code } => AgencyCommErrorKind::RevRegDefNotFound,
            _ if { error_utils::REV_DELTA_NOT_FOUND.code_num == code } => AgencyCommErrorKind::RevDeltaNotFound,
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

pub fn reset_current_error() {
    CURRENT_ERROR_C_JSON.with(|error| {
        error.replace(None);
    })
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

pub fn get_current_error_c_json() -> *const c_char {
    let mut value = ptr::null();

    CURRENT_ERROR_C_JSON.try_with(|err|
        err.borrow().as_ref().map(|err| value = err.as_ptr())
    )
        .map_err(|err| error!("Thread local variable access failed with: {:?}", err)).ok();

    value
}
