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
pub enum VcxErrorKind {
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
pub struct VcxError {
    inner: Context<VcxErrorKind>
}

impl Fail for VcxError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for VcxError {
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

impl VcxError {
    pub fn from_msg<D>(kind: VcxErrorKind, msg: D) -> VcxError
        where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
        VcxError { inner: Context::new(msg).context(kind) }
    }

    pub fn kind(&self) -> VcxErrorKind {
        *self.inner.get_context()
    }

    pub fn extend<D>(self, msg: D) -> VcxError
        where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
        let kind = self.kind();
        VcxError { inner: self.inner.map(|_| msg).context(kind) }
    }

    pub fn map<D>(self, kind: VcxErrorKind, msg: D) -> VcxError
        where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
        VcxError { inner: self.inner.map(|_| msg).context(kind) }
    }
}

pub fn err_msg<D>(kind: VcxErrorKind, msg: D) -> VcxError
    where D: fmt::Display + fmt::Debug + Send + Sync + 'static {
    VcxError::from_msg(kind, msg)
}

impl From<VcxErrorKind> for VcxError {
    fn from(kind: VcxErrorKind) -> VcxError {
        VcxError::from_msg(kind, error_utils::error_message(&kind.clone().into()))
    }
}

impl From<Context<VcxErrorKind>> for VcxError {
    fn from(inner: Context<VcxErrorKind>) -> VcxError {
        VcxError { inner }
    }
}

impl From<VcxError> for u32 {
    fn from(code: VcxError) -> u32 {
        set_current_error(&code);
        code.kind().into()
    }
}

impl From<VcxErrorKind> for u32 {
    fn from(code: VcxErrorKind) -> u32 {
        match code {
            VcxErrorKind::InvalidState => error_utils::INVALID_STATE.code_num,
            VcxErrorKind::InvalidConfiguration => error_utils::INVALID_CONFIGURATION.code_num,
            VcxErrorKind::InvalidHandle => error_utils::INVALID_OBJ_HANDLE.code_num,
            VcxErrorKind::InvalidJson => error_utils::INVALID_JSON.code_num,
            VcxErrorKind::InvalidOption => error_utils::INVALID_OPTION.code_num,
            VcxErrorKind::InvalidMessagePack => error_utils::INVALID_MSGPACK.code_num,
            VcxErrorKind::ObjectCacheError => error_utils::OBJECT_CACHE_ERROR.code_num,
            VcxErrorKind::NoPaymentInformation => error_utils::NO_PAYMENT_INFORMATION.code_num,
            VcxErrorKind::NotReady => error_utils::NOT_READY.code_num,
            VcxErrorKind::InvalidRevocationDetails => error_utils::INVALID_REVOCATION_DETAILS.code_num,
            VcxErrorKind::GeneralConnectionError => error_utils::CONNECTION_ERROR.code_num,
            VcxErrorKind::IOError => error_utils::IOERROR.code_num,
            VcxErrorKind::LibindyInvalidStructure => error_utils::LIBINDY_INVALID_STRUCTURE.code_num,
            VcxErrorKind::TimeoutLibindy => error_utils::TIMEOUT_LIBINDY_ERROR.code_num,
            VcxErrorKind::InvalidLibindyParam => error_utils::INVALID_LIBINDY_PARAM.code_num,
            VcxErrorKind::AlreadyInitialized => error_utils::ALREADY_INITIALIZED.code_num,
            VcxErrorKind::CreateConnection => error_utils::CREATE_CONNECTION_ERROR.code_num,
            VcxErrorKind::InvalidConnectionHandle => error_utils::INVALID_CONNECTION_HANDLE.code_num,
            VcxErrorKind::InvalidInviteDetail => error_utils::INVALID_INVITE_DETAILS.code_num,
            VcxErrorKind::InvalidRedirectDetail => error_utils::INVALID_REDIRECT_DETAILS.code_num,
            VcxErrorKind::DeleteConnection => error_utils::CANNOT_DELETE_CONNECTION.code_num,
            VcxErrorKind::CreateCredDef => error_utils::CREATE_CREDENTIAL_DEF_ERR.code_num,
            VcxErrorKind::CredDefAlreadyCreated => error_utils::CREDENTIAL_DEF_ALREADY_CREATED.code_num,
            VcxErrorKind::InvalidCredDefHandle => error_utils::INVALID_CREDENTIAL_DEF_HANDLE.code_num,
            VcxErrorKind::InvalidRevocationEntry => error_utils::INVALID_REV_ENTRY.code_num,
            VcxErrorKind::CreateRevRegDef => error_utils::INVALID_REV_REG_DEF_CREATION.code_num,
            VcxErrorKind::InvalidCredentialHandle => error_utils::INVALID_CREDENTIAL_HANDLE.code_num,
            VcxErrorKind::CreateCredentialRequest => error_utils::CREATE_CREDENTIAL_REQUEST_ERROR.code_num,
            VcxErrorKind::InvalidIssuerCredentialHandle => error_utils::INVALID_ISSUER_CREDENTIAL_HANDLE.code_num,
            VcxErrorKind::InvalidCredentialRequest => error_utils::INVALID_CREDENTIAL_REQUEST.code_num,
            VcxErrorKind::InvalidCredential => error_utils::INVALID_CREDENTIAL_JSON.code_num,
            VcxErrorKind::InsufficientTokenAmount => error_utils::INSUFFICIENT_TOKEN_AMOUNT.code_num,
            VcxErrorKind::InvalidProofHandle => error_utils::INVALID_PROOF_HANDLE.code_num,
            VcxErrorKind::InvalidDisclosedProofHandle => error_utils::INVALID_DISCLOSED_PROOF_HANDLE.code_num,
            VcxErrorKind::InvalidProof => error_utils::INVALID_PROOF.code_num,
            VcxErrorKind::InvalidSchema => error_utils::INVALID_SCHEMA.code_num,
            VcxErrorKind::InvalidProofCredentialData => error_utils::INVALID_PROOF_CREDENTIAL_DATA.code_num,
            VcxErrorKind::CreateProof => error_utils::CREATE_PROOF_ERROR.code_num,
            VcxErrorKind::InvalidRevocationTimestamp => error_utils::INVALID_REVOCATION_TIMESTAMP.code_num,
            VcxErrorKind::CreateSchema => error_utils::INVALID_SCHEMA_CREATION.code_num,
            VcxErrorKind::InvalidSchemaHandle => error_utils::INVALID_SCHEMA_HANDLE.code_num,
            VcxErrorKind::InvalidSchemaSeqNo => error_utils::INVALID_SCHEMA_SEQ_NO.code_num,
            VcxErrorKind::DuplicationSchema => error_utils::DUPLICATE_SCHEMA.code_num,
            VcxErrorKind::UnknownSchemaRejection => error_utils::UNKNOWN_SCHEMA_REJECTION.code_num,
            VcxErrorKind::WalletCreate => error_utils::INVALID_WALLET_CREATION.code_num,
            VcxErrorKind::MissingWalletName => error_utils::MISSING_WALLET_NAME.code_num,
            VcxErrorKind::WalletAccessFailed => error_utils::WALLET_ACCESS_FAILED.code_num,
            VcxErrorKind::InvalidWalletHandle => error_utils::INVALID_WALLET_HANDLE.code_num,
            VcxErrorKind::DuplicationWallet => error_utils::WALLET_ALREADY_EXISTS.code_num,
            VcxErrorKind::WalletNotFound => error_utils::WALLET_NOT_FOUND.code_num,
            VcxErrorKind::WalletRecordNotFound => error_utils::WALLET_RECORD_NOT_FOUND.code_num,
            VcxErrorKind::PoolLedgerConnect => error_utils::POOL_LEDGER_CONNECT.code_num,
            VcxErrorKind::InvalidGenesisTxnPath => error_utils::INVALID_GENESIS_TXN_PATH.code_num,
            VcxErrorKind::CreatePoolConfig => error_utils::CREATE_POOL_CONFIG.code_num,
            VcxErrorKind::DuplicationWalletRecord => error_utils::DUPLICATE_WALLET_RECORD.code_num,
            VcxErrorKind::WalletAlreadyOpen => error_utils::WALLET_ALREADY_OPEN.code_num,
            VcxErrorKind::DuplicationMasterSecret => error_utils::DUPLICATE_MASTER_SECRET.code_num,
            VcxErrorKind::DuplicationDid => error_utils::DID_ALREADY_EXISTS_IN_WALLET.code_num,
            VcxErrorKind::InvalidLedgerResponse => error_utils::INVALID_LEDGER_RESPONSE.code_num,
            VcxErrorKind::InvalidAttributesStructure => error_utils::INVALID_ATTRIBUTES_STRUCTURE.code_num,
            VcxErrorKind::InvalidPaymentAddress => error_utils::INVALID_PAYMENT_ADDRESS.code_num,
            VcxErrorKind::NoEndpoint => error_utils::NO_ENDPOINT.code_num,
            VcxErrorKind::InvalidProofRequest => error_utils::INVALID_PROOF_REQUEST.code_num,
            VcxErrorKind::NoPoolOpen => error_utils::NO_POOL_OPEN.code_num,
            VcxErrorKind::PostMessageFailed => error_utils::POST_MSG_FAILURE.code_num,
            VcxErrorKind::LoggingError => error_utils::LOGGING_ERROR.code_num,
            VcxErrorKind::EncodeError => error_utils::BIG_NUMBER_ERROR.code_num,
            VcxErrorKind::UnknownError => error_utils::UNKNOWN_ERROR.code_num,
            VcxErrorKind::InvalidDid => error_utils::INVALID_DID.code_num,
            VcxErrorKind::InvalidVerkey => error_utils::INVALID_VERKEY.code_num,
            VcxErrorKind::InvalidNonce => error_utils::INVALID_NONCE.code_num,
            VcxErrorKind::InvalidUrl => error_utils::INVALID_URL.code_num,
            VcxErrorKind::MissingWalletKey => error_utils::MISSING_WALLET_KEY.code_num,
            VcxErrorKind::MissingPaymentMethod => error_utils::MISSING_PAYMENT_METHOD.code_num,
            VcxErrorKind::SerializationError => error_utils::SERIALIZATION_ERROR.code_num,
            VcxErrorKind::NotBase58 => error_utils::NOT_BASE58.code_num,
            VcxErrorKind::InvalidHttpResponse => error_utils::INVALID_HTTP_RESPONSE.code_num,
            VcxErrorKind::InvalidMessages => error_utils::INVALID_MESSAGES.code_num,
            VcxErrorKind::MissingExportedWalletPath => error_utils::MISSING_EXPORTED_WALLET_PATH.code_num,
            VcxErrorKind::MissingBackupKey => error_utils::MISSING_BACKUP_KEY.code_num,
            VcxErrorKind::UnknownLibndyError => error_utils::UNKNOWN_LIBINDY_ERROR.code_num,
            VcxErrorKind::ActionNotSupported => error_utils::ACTION_NOT_SUPPORTED.code_num,
            VcxErrorKind::Common(num) => num,
            VcxErrorKind::LibndyError(num) => num,
            VcxErrorKind::NoAgentInformation => error_utils::NO_AGENT_INFO.code_num,
            VcxErrorKind::RevRegDefNotFound => error_utils::REV_REG_DEF_NOT_FOUND.code_num,
            VcxErrorKind::RevDeltaNotFound => error_utils::REV_DELTA_NOT_FOUND.code_num,
        }
    }
}

impl From<u32> for VcxErrorKind {
    fn from(code: u32) -> VcxErrorKind {
        match code {
            _ if { error_utils::INVALID_STATE.code_num == code } => VcxErrorKind::InvalidState,
            _ if { error_utils::INVALID_CONFIGURATION.code_num == code } => VcxErrorKind::InvalidConfiguration,
            _ if { error_utils::INVALID_OBJ_HANDLE.code_num == code } => VcxErrorKind::InvalidHandle,
            _ if { error_utils::INVALID_JSON.code_num == code } => VcxErrorKind::InvalidJson,
            _ if { error_utils::INVALID_OPTION.code_num == code } => VcxErrorKind::InvalidOption,
            _ if { error_utils::INVALID_MSGPACK.code_num == code } => VcxErrorKind::InvalidMessagePack,
            _ if { error_utils::OBJECT_CACHE_ERROR.code_num == code } => VcxErrorKind::ObjectCacheError,
            _ if { error_utils::NO_PAYMENT_INFORMATION.code_num == code } => VcxErrorKind::NoPaymentInformation,
            _ if { error_utils::NOT_READY.code_num == code } => VcxErrorKind::NotReady,
            _ if { error_utils::INVALID_REVOCATION_DETAILS.code_num == code } => VcxErrorKind::InvalidRevocationDetails,
            _ if { error_utils::CONNECTION_ERROR.code_num == code } => VcxErrorKind::GeneralConnectionError,
            _ if { error_utils::IOERROR.code_num == code } => VcxErrorKind::IOError,
            _ if { error_utils::LIBINDY_INVALID_STRUCTURE.code_num == code } => VcxErrorKind::LibindyInvalidStructure,
            _ if { error_utils::TIMEOUT_LIBINDY_ERROR.code_num == code } => VcxErrorKind::TimeoutLibindy,
            _ if { error_utils::INVALID_LIBINDY_PARAM.code_num == code } => VcxErrorKind::InvalidLibindyParam,
            _ if { error_utils::ALREADY_INITIALIZED.code_num == code } => VcxErrorKind::AlreadyInitialized,
            _ if { error_utils::CREATE_CONNECTION_ERROR.code_num == code } => VcxErrorKind::CreateConnection,
            _ if { error_utils::INVALID_CONNECTION_HANDLE.code_num == code } => VcxErrorKind::InvalidConnectionHandle,
            _ if { error_utils::INVALID_INVITE_DETAILS.code_num == code } => VcxErrorKind::InvalidInviteDetail,
            _ if { error_utils::INVALID_REDIRECT_DETAILS.code_num == code } => VcxErrorKind::InvalidRedirectDetail,
            _ if { error_utils::CANNOT_DELETE_CONNECTION.code_num == code } => VcxErrorKind::DeleteConnection,
            _ if { error_utils::CREATE_CREDENTIAL_DEF_ERR.code_num == code } => VcxErrorKind::CreateCredDef,
            _ if { error_utils::CREDENTIAL_DEF_ALREADY_CREATED.code_num == code } => VcxErrorKind::CredDefAlreadyCreated,
            _ if { error_utils::INVALID_CREDENTIAL_DEF_HANDLE.code_num == code } => VcxErrorKind::InvalidCredDefHandle,
            _ if { error_utils::INVALID_REV_ENTRY.code_num == code } => VcxErrorKind::InvalidRevocationEntry,
            _ if { error_utils::INVALID_REV_REG_DEF_CREATION.code_num == code } => VcxErrorKind::CreateRevRegDef,
            _ if { error_utils::INVALID_CREDENTIAL_HANDLE.code_num == code } => VcxErrorKind::InvalidCredentialHandle,
            _ if { error_utils::CREATE_CREDENTIAL_REQUEST_ERROR.code_num == code } => VcxErrorKind::CreateCredentialRequest,
            _ if { error_utils::INVALID_ISSUER_CREDENTIAL_HANDLE.code_num == code } => VcxErrorKind::InvalidIssuerCredentialHandle,
            _ if { error_utils::INVALID_CREDENTIAL_REQUEST.code_num == code } => VcxErrorKind::InvalidCredentialRequest,
            _ if { error_utils::INVALID_CREDENTIAL_JSON.code_num == code } => VcxErrorKind::InvalidCredential,
            _ if { error_utils::INSUFFICIENT_TOKEN_AMOUNT.code_num == code } => VcxErrorKind::InsufficientTokenAmount,
            _ if { error_utils::INVALID_PROOF_HANDLE.code_num == code } => VcxErrorKind::InvalidProofHandle,
            _ if { error_utils::INVALID_DISCLOSED_PROOF_HANDLE.code_num == code } => VcxErrorKind::InvalidDisclosedProofHandle,
            _ if { error_utils::INVALID_PROOF.code_num == code } => VcxErrorKind::InvalidProof,
            _ if { error_utils::INVALID_SCHEMA.code_num == code } => VcxErrorKind::InvalidSchema,
            _ if { error_utils::INVALID_PROOF_CREDENTIAL_DATA.code_num == code } => VcxErrorKind::InvalidProofCredentialData,
            _ if { error_utils::CREATE_PROOF_ERROR.code_num == code } => VcxErrorKind::CreateProof,
            _ if { error_utils::INVALID_REVOCATION_TIMESTAMP.code_num == code } => VcxErrorKind::InvalidRevocationTimestamp,
            _ if { error_utils::INVALID_SCHEMA_CREATION.code_num == code } => VcxErrorKind::CreateSchema,
            _ if { error_utils::INVALID_SCHEMA_HANDLE.code_num == code } => VcxErrorKind::InvalidSchemaHandle,
            _ if { error_utils::INVALID_SCHEMA_SEQ_NO.code_num == code } => VcxErrorKind::InvalidSchemaSeqNo,
            _ if { error_utils::DUPLICATE_SCHEMA.code_num == code } => VcxErrorKind::DuplicationSchema,
            _ if { error_utils::UNKNOWN_SCHEMA_REJECTION.code_num == code } => VcxErrorKind::UnknownSchemaRejection,
            _ if { error_utils::INVALID_WALLET_CREATION.code_num == code } => VcxErrorKind::WalletCreate,
            _ if { error_utils::MISSING_WALLET_NAME.code_num == code } => VcxErrorKind::MissingWalletName,
            _ if { error_utils::WALLET_ACCESS_FAILED.code_num == code } => VcxErrorKind::WalletAccessFailed,
            _ if { error_utils::INVALID_WALLET_HANDLE.code_num == code } => VcxErrorKind::InvalidWalletHandle,
            _ if { error_utils::WALLET_ALREADY_EXISTS.code_num == code } => VcxErrorKind::DuplicationWallet,
            _ if { error_utils::WALLET_NOT_FOUND.code_num == code } => VcxErrorKind::WalletNotFound,
            _ if { error_utils::WALLET_RECORD_NOT_FOUND.code_num == code } => VcxErrorKind::WalletRecordNotFound,
            _ if { error_utils::POOL_LEDGER_CONNECT.code_num == code } => VcxErrorKind::PoolLedgerConnect,
            _ if { error_utils::INVALID_GENESIS_TXN_PATH.code_num == code } => VcxErrorKind::InvalidGenesisTxnPath,
            _ if { error_utils::CREATE_POOL_CONFIG.code_num == code } => VcxErrorKind::CreatePoolConfig,
            _ if { error_utils::DUPLICATE_WALLET_RECORD.code_num == code } => VcxErrorKind::DuplicationWalletRecord,
            _ if { error_utils::WALLET_ALREADY_OPEN.code_num == code } => VcxErrorKind::WalletAlreadyOpen,
            _ if { error_utils::DUPLICATE_MASTER_SECRET.code_num == code } => VcxErrorKind::DuplicationMasterSecret,
            _ if { error_utils::DID_ALREADY_EXISTS_IN_WALLET.code_num == code } => VcxErrorKind::DuplicationDid,
            _ if { error_utils::INVALID_LEDGER_RESPONSE.code_num == code } => VcxErrorKind::InvalidLedgerResponse,
            _ if { error_utils::INVALID_ATTRIBUTES_STRUCTURE.code_num == code } => VcxErrorKind::InvalidAttributesStructure,
            _ if { error_utils::INVALID_PAYMENT_ADDRESS.code_num == code } => VcxErrorKind::InvalidPaymentAddress,
            _ if { error_utils::NO_ENDPOINT.code_num == code } => VcxErrorKind::NoEndpoint,
            _ if { error_utils::INVALID_PROOF_REQUEST.code_num == code } => VcxErrorKind::InvalidProofRequest,
            _ if { error_utils::NO_POOL_OPEN.code_num == code } => VcxErrorKind::NoPoolOpen,
            _ if { error_utils::POST_MSG_FAILURE.code_num == code } => VcxErrorKind::PostMessageFailed,
            _ if { error_utils::LOGGING_ERROR.code_num == code } => VcxErrorKind::LoggingError,
            _ if { error_utils::BIG_NUMBER_ERROR.code_num == code } => VcxErrorKind::EncodeError,
            _ if { error_utils::UNKNOWN_ERROR.code_num == code } => VcxErrorKind::UnknownError,
            _ if { error_utils::INVALID_DID.code_num == code } => VcxErrorKind::InvalidDid,
            _ if { error_utils::INVALID_VERKEY.code_num == code } => VcxErrorKind::InvalidVerkey,
            _ if { error_utils::INVALID_NONCE.code_num == code } => VcxErrorKind::InvalidNonce,
            _ if { error_utils::INVALID_URL.code_num == code } => VcxErrorKind::InvalidUrl,
            _ if { error_utils::MISSING_WALLET_KEY.code_num == code } => VcxErrorKind::MissingWalletKey,
            _ if { error_utils::MISSING_PAYMENT_METHOD.code_num == code } => VcxErrorKind::MissingPaymentMethod,
            _ if { error_utils::SERIALIZATION_ERROR.code_num == code } => VcxErrorKind::SerializationError,
            _ if { error_utils::NOT_BASE58.code_num == code } => VcxErrorKind::NotBase58,
            _ if { error_utils::INVALID_HTTP_RESPONSE.code_num == code } => VcxErrorKind::InvalidHttpResponse,
            _ if { error_utils::INVALID_MESSAGES.code_num == code } => VcxErrorKind::InvalidMessages,
            _ if { error_utils::MISSING_EXPORTED_WALLET_PATH.code_num == code } => VcxErrorKind::MissingExportedWalletPath,
            _ if { error_utils::MISSING_BACKUP_KEY.code_num == code } => VcxErrorKind::MissingBackupKey,
            _ if { error_utils::UNKNOWN_LIBINDY_ERROR.code_num == code } => VcxErrorKind::UnknownLibndyError,
            _ if { error_utils::ACTION_NOT_SUPPORTED.code_num == code } => VcxErrorKind::ActionNotSupported,
            _ if { error_utils::NO_AGENT_INFO.code_num == code } => VcxErrorKind::NoAgentInformation,
            _ if { error_utils::REV_REG_DEF_NOT_FOUND.code_num == code } => VcxErrorKind::RevRegDefNotFound,
            _ if { error_utils::REV_DELTA_NOT_FOUND.code_num == code } => VcxErrorKind::RevDeltaNotFound,
            _ => VcxErrorKind::UnknownError,
        }
    }
}

impl From<IndyError> for VcxError {
    fn from(error: IndyError) -> Self {
        match error.error_code as u32 {
            100..=111 => VcxError::from_msg(VcxErrorKind::InvalidLibindyParam, error.message),
            113 => VcxError::from_msg(VcxErrorKind::LibindyInvalidStructure, error.message),
            114 => VcxError::from_msg(VcxErrorKind::IOError, error.message),
            200 => VcxError::from_msg(VcxErrorKind::InvalidWalletHandle, error.message),
            203 => VcxError::from_msg(VcxErrorKind::DuplicationWallet, error.message),
            204 => VcxError::from_msg(VcxErrorKind::WalletNotFound, error.message),
            206 => VcxError::from_msg(VcxErrorKind::WalletAlreadyOpen, error.message),
            212 => VcxError::from_msg(VcxErrorKind::WalletRecordNotFound, error.message),
            213 => VcxError::from_msg(VcxErrorKind::DuplicationWalletRecord, error.message),
            306 => VcxError::from_msg(VcxErrorKind::CreatePoolConfig, error.message),
            404 => VcxError::from_msg(VcxErrorKind::DuplicationMasterSecret, error.message),
            407 => VcxError::from_msg(VcxErrorKind::CredDefAlreadyCreated, error.message),
            600 => VcxError::from_msg(VcxErrorKind::DuplicationDid, error.message),
            702 => VcxError::from_msg(VcxErrorKind::InsufficientTokenAmount, error.message),
            error_code => VcxError::from_msg(VcxErrorKind::LibndyError(error_code), error.message)
        }
    }
}

pub type VcxResult<T> = Result<T, VcxError>;

/// Extension methods for `Result`.
pub trait VcxResultExt<T, E> {
    fn to_vcx<D>(self, kind: VcxErrorKind, msg: D) -> VcxResult<T> where D: fmt::Display + Send + Sync + 'static;
}

impl<T, E> VcxResultExt<T, E> for Result<T, E> where E: Fail
{
    fn to_vcx<D>(self, kind: VcxErrorKind, msg: D) -> VcxResult<T> where D: fmt::Display + Send + Sync + 'static {
        self.map_err(|err| err.context(msg).context(kind).into())
    }
}

/// Extension methods for `Error`.
pub trait VcxErrorExt {
    fn to_vcx<D>(self, kind: VcxErrorKind, msg: D) -> VcxError where D: fmt::Display + Send + Sync + 'static;
}

impl<E> VcxErrorExt for E where E: Fail
{
    fn to_vcx<D>(self, kind: VcxErrorKind, msg: D) -> VcxError where D: fmt::Display + Send + Sync + 'static {
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

pub fn set_current_error(err: &VcxError) {
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
