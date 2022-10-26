use std::fmt;
use std::sync;

use failure::{Backtrace, Context, Fail};

use crate::utils::error;

pub mod prelude {
    pub use super::{err_msg, MessagesError, MessagesErrorExt, MessagesResult, MessagesResultExt, MesssagesErrorKind};
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum MesssagesErrorKind {
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
    #[fail(
        display = "No revocation delta found in storage for this revocation registry. Were any credentials locally revoked?"
    )]
    RevDeltaNotFound,
    #[fail(display = "Failed to clean stored revocation delta")]
    RevDeltaFailedToClear,

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

    // Public agent
    #[fail(display = "Could not create public agent")]
    CreatePublicAgent,

    // Out of Band
    #[fail(display = "Could not create out of band message.")]
    CreateOutOfBand,

    // Pool
    #[fail(display = "Invalid genesis transactions path.")]
    InvalidGenesisTxnPath,
    #[fail(display = "Formatting for Pool Config are incorrect.")]
    CreatePoolConfig,
    #[fail(display = "Connection to Pool Ledger.")]
    PoolLedgerConnect,
    #[fail(display = "Ledger rejected submitted request.")]
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
    #[fail(display = "Error creating agent in agency")]
    CreateAgent,

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
pub struct MessagesError {
    inner: Context<MesssagesErrorKind>,
}

impl Fail for MessagesError {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for MessagesError {
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

impl MessagesError {
    pub fn from_msg<D>(kind: MesssagesErrorKind, msg: D) -> MessagesError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        MessagesError {
            inner: Context::new(msg).context(kind),
        }
    }

    pub fn kind(&self) -> MesssagesErrorKind {
        *self.inner.get_context()
    }

    pub fn extend<D>(self, msg: D) -> MessagesError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        let kind = self.kind();
        MessagesError {
            inner: self.inner.map(|_| msg).context(kind),
        }
    }

    pub fn map<D>(self, kind: MesssagesErrorKind, msg: D) -> MessagesError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        MessagesError {
            inner: self.inner.map(|_| msg).context(kind),
        }
    }
}

pub fn err_msg<D>(kind: MesssagesErrorKind, msg: D) -> MessagesError
where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    MessagesError::from_msg(kind, msg)
}

impl From<MesssagesErrorKind> for MessagesError {
    fn from(kind: MesssagesErrorKind) -> MessagesError {
        MessagesError::from_msg(kind, crate::utils::error::error_message(&kind.into()))
    }
}

impl<T> From<sync::PoisonError<T>> for MessagesError {
    fn from(_: sync::PoisonError<T>) -> Self {
        MessagesError {
            inner: Context::new(Backtrace::new()).context(MesssagesErrorKind::PoisonedLock),
        }
    }
}

impl From<Context<MesssagesErrorKind>> for MessagesError {
    fn from(inner: Context<MesssagesErrorKind>) -> MessagesError {
        MessagesError { inner }
    }
}

pub type MessagesResult<T> = Result<T, MessagesError>;

/// Extension methods for `Result`.
pub trait MessagesResultExt<T, E> {
    fn to_vcx<D>(self, kind: MesssagesErrorKind, msg: D) -> MessagesResult<T>
    where
        D: fmt::Display + Send + Sync + 'static;
}

impl<T, E> MessagesResultExt<T, E> for Result<T, E>
where
    E: Fail,
{
    fn to_vcx<D>(self, kind: MesssagesErrorKind, msg: D) -> MessagesResult<T>
    where
        D: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| err.context(msg).context(kind).into())
    }
}

/// Extension methods for `Error`.
pub trait MessagesErrorExt {
    fn to_vcx<D>(self, kind: MesssagesErrorKind, msg: D) -> MessagesError
    where
        D: fmt::Display + Send + Sync + 'static;
}

impl<E> MessagesErrorExt for E
where
    E: Fail,
{
    fn to_vcx<D>(self, kind: MesssagesErrorKind, msg: D) -> MessagesError
    where
        D: fmt::Display + Send + Sync + 'static,
    {
        self.context(format!("\n{}: {}", std::any::type_name::<E>(), msg))
            .context(kind)
            .into()
    }
}

impl From<MessagesError> for u32 {
    fn from(code: MessagesError) -> u32 {
        code.kind().into()
    }
}

impl From<MesssagesErrorKind> for u32 {
    fn from(code: MesssagesErrorKind) -> u32 {
        match code {
            MesssagesErrorKind::InvalidState => error::INVALID_STATE.code_num,
            MesssagesErrorKind::InvalidConfiguration => error::INVALID_CONFIGURATION.code_num,
            MesssagesErrorKind::InvalidHandle => error::INVALID_OBJ_HANDLE.code_num,
            MesssagesErrorKind::InvalidJson => error::INVALID_JSON.code_num,
            MesssagesErrorKind::InvalidOption => error::INVALID_OPTION.code_num,
            MesssagesErrorKind::InvalidMessagePack => error::INVALID_MSGPACK.code_num,
            MesssagesErrorKind::ObjectCacheError => error::OBJECT_CACHE_ERROR.code_num,
            MesssagesErrorKind::NoPaymentInformation => error::NO_PAYMENT_INFORMATION.code_num,
            MesssagesErrorKind::NotReady => error::NOT_READY.code_num,
            MesssagesErrorKind::InvalidRevocationDetails => error::INVALID_REVOCATION_DETAILS.code_num,
            MesssagesErrorKind::GeneralConnectionError => error::CONNECTION_ERROR.code_num,
            MesssagesErrorKind::IOError => error::IOERROR.code_num,
            MesssagesErrorKind::LibindyInvalidStructure => error::LIBINDY_INVALID_STRUCTURE.code_num,
            MesssagesErrorKind::TimeoutLibindy => error::TIMEOUT_LIBINDY_ERROR.code_num,
            MesssagesErrorKind::InvalidLibindyParam => error::INVALID_LIBINDY_PARAM.code_num,
            MesssagesErrorKind::AlreadyInitialized => error::ALREADY_INITIALIZED.code_num,
            MesssagesErrorKind::CreateConnection => error::CREATE_CONNECTION_ERROR.code_num,
            MesssagesErrorKind::InvalidConnectionHandle => error::INVALID_CONNECTION_HANDLE.code_num,
            MesssagesErrorKind::InvalidInviteDetail => error::INVALID_INVITE_DETAILS.code_num,
            MesssagesErrorKind::InvalidRedirectDetail => error::INVALID_REDIRECT_DETAILS.code_num,
            MesssagesErrorKind::DeleteConnection => error::CANNOT_DELETE_CONNECTION.code_num,
            MesssagesErrorKind::CreateCredDef => error::CREATE_CREDENTIAL_DEF_ERR.code_num,
            MesssagesErrorKind::CredDefAlreadyCreated => error::CREDENTIAL_DEF_ALREADY_CREATED.code_num,
            MesssagesErrorKind::InvalidCredDefHandle => error::INVALID_CREDENTIAL_DEF_HANDLE.code_num,
            MesssagesErrorKind::InvalidRevocationEntry => error::INVALID_REV_ENTRY.code_num,
            MesssagesErrorKind::CreateRevRegDef => error::INVALID_REV_REG_DEF_CREATION.code_num,
            MesssagesErrorKind::InvalidCredentialHandle => error::INVALID_CREDENTIAL_HANDLE.code_num,
            MesssagesErrorKind::CreateCredentialRequest => error::CREATE_CREDENTIAL_REQUEST_ERROR.code_num,
            MesssagesErrorKind::InvalidIssuerCredentialHandle => error::INVALID_ISSUER_CREDENTIAL_HANDLE.code_num,
            MesssagesErrorKind::InvalidCredentialRequest => error::INVALID_CREDENTIAL_REQUEST.code_num,
            MesssagesErrorKind::InvalidCredential => error::INVALID_CREDENTIAL_JSON.code_num,
            MesssagesErrorKind::InsufficientTokenAmount => error::INSUFFICIENT_TOKEN_AMOUNT.code_num,
            MesssagesErrorKind::InvalidProofHandle => error::INVALID_PROOF_HANDLE.code_num,
            MesssagesErrorKind::InvalidDisclosedProofHandle => error::INVALID_DISCLOSED_PROOF_HANDLE.code_num,
            MesssagesErrorKind::InvalidProof => error::INVALID_PROOF.code_num,
            MesssagesErrorKind::InvalidSchema => error::INVALID_SCHEMA.code_num,
            MesssagesErrorKind::InvalidProofCredentialData => error::INVALID_PROOF_CREDENTIAL_DATA.code_num,
            MesssagesErrorKind::CreateProof => error::CREATE_PROOF_ERROR.code_num,
            MesssagesErrorKind::InvalidRevocationTimestamp => error::INVALID_REVOCATION_TIMESTAMP.code_num,
            MesssagesErrorKind::CreateSchema => error::INVALID_SCHEMA_CREATION.code_num,
            MesssagesErrorKind::InvalidSchemaHandle => error::INVALID_SCHEMA_HANDLE.code_num,
            MesssagesErrorKind::InvalidSchemaSeqNo => error::INVALID_SCHEMA_SEQ_NO.code_num,
            MesssagesErrorKind::DuplicationSchema => error::DUPLICATE_SCHEMA.code_num,
            MesssagesErrorKind::UnknownSchemaRejection => error::UNKNOWN_SCHEMA_REJECTION.code_num,
            MesssagesErrorKind::WalletCreate => error::INVALID_WALLET_CREATION.code_num,
            MesssagesErrorKind::MissingWalletName => error::MISSING_WALLET_NAME.code_num,
            MesssagesErrorKind::WalletAccessFailed => error::WALLET_ACCESS_FAILED.code_num,
            MesssagesErrorKind::InvalidWalletHandle => error::INVALID_WALLET_HANDLE.code_num,
            MesssagesErrorKind::DuplicationWallet => error::WALLET_ALREADY_EXISTS.code_num,
            MesssagesErrorKind::WalletNotFound => error::WALLET_NOT_FOUND.code_num,
            MesssagesErrorKind::WalletRecordNotFound => error::WALLET_RECORD_NOT_FOUND.code_num,
            MesssagesErrorKind::PoolLedgerConnect => error::POOL_LEDGER_CONNECT.code_num,
            MesssagesErrorKind::InvalidGenesisTxnPath => error::INVALID_GENESIS_TXN_PATH.code_num,
            MesssagesErrorKind::CreatePoolConfig => error::CREATE_POOL_CONFIG.code_num,
            MesssagesErrorKind::DuplicationWalletRecord => error::DUPLICATE_WALLET_RECORD.code_num,
            MesssagesErrorKind::WalletAlreadyOpen => error::WALLET_ALREADY_OPEN.code_num,
            MesssagesErrorKind::DuplicationMasterSecret => error::DUPLICATE_MASTER_SECRET.code_num,
            MesssagesErrorKind::DuplicationDid => error::DID_ALREADY_EXISTS_IN_WALLET.code_num,
            MesssagesErrorKind::InvalidLedgerResponse => error::INVALID_LEDGER_RESPONSE.code_num,
            MesssagesErrorKind::InvalidAttributesStructure => error::INVALID_ATTRIBUTES_STRUCTURE.code_num,
            MesssagesErrorKind::InvalidPaymentAddress => error::INVALID_PAYMENT_ADDRESS.code_num,
            MesssagesErrorKind::NoEndpoint => error::NO_ENDPOINT.code_num,
            MesssagesErrorKind::InvalidProofRequest => error::INVALID_PROOF_REQUEST.code_num,
            MesssagesErrorKind::NoPoolOpen => error::NO_POOL_OPEN.code_num,
            MesssagesErrorKind::PostMessageFailed => error::POST_MSG_FAILURE.code_num,
            MesssagesErrorKind::LoggingError => error::LOGGING_ERROR.code_num,
            MesssagesErrorKind::EncodeError => error::BIG_NUMBER_ERROR.code_num,
            MesssagesErrorKind::UnknownError => error::UNKNOWN_ERROR.code_num,
            MesssagesErrorKind::InvalidDid => error::INVALID_DID.code_num,
            MesssagesErrorKind::InvalidVerkey => error::INVALID_VERKEY.code_num,
            MesssagesErrorKind::InvalidNonce => error::INVALID_NONCE.code_num,
            MesssagesErrorKind::InvalidUrl => error::INVALID_URL.code_num,
            MesssagesErrorKind::MissingWalletKey => error::MISSING_WALLET_KEY.code_num,
            MesssagesErrorKind::MissingPaymentMethod => error::MISSING_PAYMENT_METHOD.code_num,
            MesssagesErrorKind::SerializationError => error::SERIALIZATION_ERROR.code_num,
            MesssagesErrorKind::NotBase58 => error::NOT_BASE58.code_num,
            MesssagesErrorKind::InvalidHttpResponse => error::INVALID_HTTP_RESPONSE.code_num,
            MesssagesErrorKind::InvalidMessages => error::INVALID_MESSAGES.code_num,
            MesssagesErrorKind::MissingExportedWalletPath => error::MISSING_EXPORTED_WALLET_PATH.code_num,
            MesssagesErrorKind::MissingBackupKey => error::MISSING_BACKUP_KEY.code_num,
            MesssagesErrorKind::UnknownLibndyError => error::UNKNOWN_LIBINDY_ERROR.code_num,
            MesssagesErrorKind::ActionNotSupported => error::ACTION_NOT_SUPPORTED.code_num,
            MesssagesErrorKind::Common(num) => num,
            MesssagesErrorKind::LibndyError(num) => num,
            MesssagesErrorKind::NoAgentInformation => error::NO_AGENT_INFO.code_num,
            MesssagesErrorKind::RevRegDefNotFound => error::REV_REG_DEF_NOT_FOUND.code_num,
            MesssagesErrorKind::RevDeltaNotFound => error::REV_DELTA_NOT_FOUND.code_num,
            MesssagesErrorKind::RevDeltaFailedToClear => error::REV_DELTA_FAILED_TO_CLEAR.code_num,
            MesssagesErrorKind::PoisonedLock => error::POISONED_LOCK.code_num,
            MesssagesErrorKind::InvalidMessageFormat => error::INVALID_MESSAGE_FORMAT.code_num,
            MesssagesErrorKind::CreatePublicAgent => error::CREATE_PUBLIC_AGENT.code_num,
            MesssagesErrorKind::CreateOutOfBand => error::CREATE_OUT_OF_BAND.code_num,
            MesssagesErrorKind::CreateAgent => error::CREATE_AGENT.code_num,
        }
    }
}

impl From<u32> for MesssagesErrorKind {
    fn from(code: u32) -> MesssagesErrorKind {
        match code {
            _ if { error::INVALID_STATE.code_num == code } => MesssagesErrorKind::InvalidState,
            _ if { error::INVALID_CONFIGURATION.code_num == code } => MesssagesErrorKind::InvalidConfiguration,
            _ if { error::INVALID_OBJ_HANDLE.code_num == code } => MesssagesErrorKind::InvalidHandle,
            _ if { error::INVALID_JSON.code_num == code } => MesssagesErrorKind::InvalidJson,
            _ if { error::INVALID_OPTION.code_num == code } => MesssagesErrorKind::InvalidOption,
            _ if { error::INVALID_MSGPACK.code_num == code } => MesssagesErrorKind::InvalidMessagePack,
            _ if { error::OBJECT_CACHE_ERROR.code_num == code } => MesssagesErrorKind::ObjectCacheError,
            _ if { error::NO_PAYMENT_INFORMATION.code_num == code } => MesssagesErrorKind::NoPaymentInformation,
            _ if { error::NOT_READY.code_num == code } => MesssagesErrorKind::NotReady,
            _ if { error::INVALID_REVOCATION_DETAILS.code_num == code } => MesssagesErrorKind::InvalidRevocationDetails,
            _ if { error::CONNECTION_ERROR.code_num == code } => MesssagesErrorKind::GeneralConnectionError,
            _ if { error::IOERROR.code_num == code } => MesssagesErrorKind::IOError,
            _ if { error::LIBINDY_INVALID_STRUCTURE.code_num == code } => MesssagesErrorKind::LibindyInvalidStructure,
            _ if { error::TIMEOUT_LIBINDY_ERROR.code_num == code } => MesssagesErrorKind::TimeoutLibindy,
            _ if { error::INVALID_LIBINDY_PARAM.code_num == code } => MesssagesErrorKind::InvalidLibindyParam,
            _ if { error::ALREADY_INITIALIZED.code_num == code } => MesssagesErrorKind::AlreadyInitialized,
            _ if { error::CREATE_CONNECTION_ERROR.code_num == code } => MesssagesErrorKind::CreateConnection,
            _ if { error::INVALID_CONNECTION_HANDLE.code_num == code } => MesssagesErrorKind::InvalidConnectionHandle,
            _ if { error::INVALID_INVITE_DETAILS.code_num == code } => MesssagesErrorKind::InvalidInviteDetail,
            _ if { error::INVALID_REDIRECT_DETAILS.code_num == code } => MesssagesErrorKind::InvalidRedirectDetail,
            _ if { error::CANNOT_DELETE_CONNECTION.code_num == code } => MesssagesErrorKind::DeleteConnection,
            _ if { error::CREATE_CREDENTIAL_DEF_ERR.code_num == code } => MesssagesErrorKind::CreateCredDef,
            _ if { error::CREDENTIAL_DEF_ALREADY_CREATED.code_num == code } => {
                MesssagesErrorKind::CredDefAlreadyCreated
            }
            _ if { error::INVALID_CREDENTIAL_DEF_HANDLE.code_num == code } => MesssagesErrorKind::InvalidCredDefHandle,
            _ if { error::INVALID_REV_ENTRY.code_num == code } => MesssagesErrorKind::InvalidRevocationEntry,
            _ if { error::INVALID_REV_REG_DEF_CREATION.code_num == code } => MesssagesErrorKind::CreateRevRegDef,
            _ if { error::INVALID_CREDENTIAL_HANDLE.code_num == code } => MesssagesErrorKind::InvalidCredentialHandle,
            _ if { error::CREATE_CREDENTIAL_REQUEST_ERROR.code_num == code } => {
                MesssagesErrorKind::CreateCredentialRequest
            }
            _ if { error::INVALID_ISSUER_CREDENTIAL_HANDLE.code_num == code } => {
                MesssagesErrorKind::InvalidIssuerCredentialHandle
            }
            _ if { error::INVALID_CREDENTIAL_REQUEST.code_num == code } => MesssagesErrorKind::InvalidCredentialRequest,
            _ if { error::INVALID_CREDENTIAL_JSON.code_num == code } => MesssagesErrorKind::InvalidCredential,
            _ if { error::INSUFFICIENT_TOKEN_AMOUNT.code_num == code } => MesssagesErrorKind::InsufficientTokenAmount,
            _ if { error::INVALID_PROOF_HANDLE.code_num == code } => MesssagesErrorKind::InvalidProofHandle,
            _ if { error::INVALID_DISCLOSED_PROOF_HANDLE.code_num == code } => {
                MesssagesErrorKind::InvalidDisclosedProofHandle
            }
            _ if { error::INVALID_PROOF.code_num == code } => MesssagesErrorKind::InvalidProof,
            _ if { error::INVALID_SCHEMA.code_num == code } => MesssagesErrorKind::InvalidSchema,
            _ if { error::INVALID_PROOF_CREDENTIAL_DATA.code_num == code } => {
                MesssagesErrorKind::InvalidProofCredentialData
            }
            _ if { error::CREATE_PROOF_ERROR.code_num == code } => MesssagesErrorKind::CreateProof,
            _ if { error::INVALID_REVOCATION_TIMESTAMP.code_num == code } => {
                MesssagesErrorKind::InvalidRevocationTimestamp
            }
            _ if { error::INVALID_SCHEMA_CREATION.code_num == code } => MesssagesErrorKind::CreateSchema,
            _ if { error::INVALID_SCHEMA_HANDLE.code_num == code } => MesssagesErrorKind::InvalidSchemaHandle,
            _ if { error::INVALID_SCHEMA_SEQ_NO.code_num == code } => MesssagesErrorKind::InvalidSchemaSeqNo,
            _ if { error::DUPLICATE_SCHEMA.code_num == code } => MesssagesErrorKind::DuplicationSchema,
            _ if { error::UNKNOWN_SCHEMA_REJECTION.code_num == code } => MesssagesErrorKind::UnknownSchemaRejection,
            _ if { error::INVALID_WALLET_CREATION.code_num == code } => MesssagesErrorKind::WalletCreate,
            _ if { error::MISSING_WALLET_NAME.code_num == code } => MesssagesErrorKind::MissingWalletName,
            _ if { error::WALLET_ACCESS_FAILED.code_num == code } => MesssagesErrorKind::WalletAccessFailed,
            _ if { error::INVALID_WALLET_HANDLE.code_num == code } => MesssagesErrorKind::InvalidWalletHandle,
            _ if { error::WALLET_ALREADY_EXISTS.code_num == code } => MesssagesErrorKind::DuplicationWallet,
            _ if { error::WALLET_NOT_FOUND.code_num == code } => MesssagesErrorKind::WalletNotFound,
            _ if { error::WALLET_RECORD_NOT_FOUND.code_num == code } => MesssagesErrorKind::WalletRecordNotFound,
            _ if { error::POOL_LEDGER_CONNECT.code_num == code } => MesssagesErrorKind::PoolLedgerConnect,
            _ if { error::INVALID_GENESIS_TXN_PATH.code_num == code } => MesssagesErrorKind::InvalidGenesisTxnPath,
            _ if { error::CREATE_POOL_CONFIG.code_num == code } => MesssagesErrorKind::CreatePoolConfig,
            _ if { error::DUPLICATE_WALLET_RECORD.code_num == code } => MesssagesErrorKind::DuplicationWalletRecord,
            _ if { error::WALLET_ALREADY_OPEN.code_num == code } => MesssagesErrorKind::WalletAlreadyOpen,
            _ if { error::DUPLICATE_MASTER_SECRET.code_num == code } => MesssagesErrorKind::DuplicationMasterSecret,
            _ if { error::DID_ALREADY_EXISTS_IN_WALLET.code_num == code } => MesssagesErrorKind::DuplicationDid,
            _ if { error::INVALID_LEDGER_RESPONSE.code_num == code } => MesssagesErrorKind::InvalidLedgerResponse,
            _ if { error::INVALID_ATTRIBUTES_STRUCTURE.code_num == code } => {
                MesssagesErrorKind::InvalidAttributesStructure
            }
            _ if { error::INVALID_PAYMENT_ADDRESS.code_num == code } => MesssagesErrorKind::InvalidPaymentAddress,
            _ if { error::NO_ENDPOINT.code_num == code } => MesssagesErrorKind::NoEndpoint,
            _ if { error::INVALID_PROOF_REQUEST.code_num == code } => MesssagesErrorKind::InvalidProofRequest,
            _ if { error::NO_POOL_OPEN.code_num == code } => MesssagesErrorKind::NoPoolOpen,
            _ if { error::POST_MSG_FAILURE.code_num == code } => MesssagesErrorKind::PostMessageFailed,
            _ if { error::LOGGING_ERROR.code_num == code } => MesssagesErrorKind::LoggingError,
            _ if { error::BIG_NUMBER_ERROR.code_num == code } => MesssagesErrorKind::EncodeError,
            _ if { error::UNKNOWN_ERROR.code_num == code } => MesssagesErrorKind::UnknownError,
            _ if { error::INVALID_DID.code_num == code } => MesssagesErrorKind::InvalidDid,
            _ if { error::INVALID_VERKEY.code_num == code } => MesssagesErrorKind::InvalidVerkey,
            _ if { error::INVALID_NONCE.code_num == code } => MesssagesErrorKind::InvalidNonce,
            _ if { error::INVALID_URL.code_num == code } => MesssagesErrorKind::InvalidUrl,
            _ if { error::MISSING_WALLET_KEY.code_num == code } => MesssagesErrorKind::MissingWalletKey,
            _ if { error::MISSING_PAYMENT_METHOD.code_num == code } => MesssagesErrorKind::MissingPaymentMethod,
            _ if { error::SERIALIZATION_ERROR.code_num == code } => MesssagesErrorKind::SerializationError,
            _ if { error::NOT_BASE58.code_num == code } => MesssagesErrorKind::NotBase58,
            _ if { error::INVALID_HTTP_RESPONSE.code_num == code } => MesssagesErrorKind::InvalidHttpResponse,
            _ if { error::INVALID_MESSAGES.code_num == code } => MesssagesErrorKind::InvalidMessages,
            _ if { error::MISSING_EXPORTED_WALLET_PATH.code_num == code } => {
                MesssagesErrorKind::MissingExportedWalletPath
            }
            _ if { error::MISSING_BACKUP_KEY.code_num == code } => MesssagesErrorKind::MissingBackupKey,
            _ if { error::UNKNOWN_LIBINDY_ERROR.code_num == code } => MesssagesErrorKind::UnknownLibndyError,
            _ if { error::ACTION_NOT_SUPPORTED.code_num == code } => MesssagesErrorKind::ActionNotSupported,
            _ if { error::NO_AGENT_INFO.code_num == code } => MesssagesErrorKind::NoAgentInformation,
            _ if { error::REV_REG_DEF_NOT_FOUND.code_num == code } => MesssagesErrorKind::RevRegDefNotFound,
            _ if { error::REV_DELTA_NOT_FOUND.code_num == code } => MesssagesErrorKind::RevDeltaNotFound,
            _ if { error::CREATE_PUBLIC_AGENT.code_num == code } => MesssagesErrorKind::CreatePublicAgent,
            _ if { error::CREATE_OUT_OF_BAND.code_num == code } => MesssagesErrorKind::CreateOutOfBand,
            _ if { error::POISONED_LOCK.code_num == code } => MesssagesErrorKind::PoisonedLock,
            _ if { error::INVALID_MESSAGE_FORMAT.code_num == code } => MesssagesErrorKind::InvalidMessageFormat,
            _ if { error::CREATE_OUT_OF_BAND.code_num == code } => MesssagesErrorKind::CreateOutOfBand,
            _ if { error::CREATE_AGENT.code_num == code } => MesssagesErrorKind::CreateAgent,
            _ if { error::REV_DELTA_FAILED_TO_CLEAR.code_num == code } => MesssagesErrorKind::RevDeltaFailedToClear,
            _ => MesssagesErrorKind::UnknownError,
        }
    }
}
