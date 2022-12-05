use std::{fmt, sync};
use std::error::Error;

use thiserror;

use agency_client;
use agency_client::error::AgencyClientErrorKind;
use messages;
use messages::error::MesssagesErrorKind as MessagesErrorKind;
use crate::utils::error;
use crate::protocols::revocation_notification::sender::state_machine::SenderConfigBuilderError;

use vdrtools::types;

pub mod prelude {
    pub use super::{err_msg, VcxError, VcxErrorKind, VcxResult};
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, thiserror::Error)]
pub enum VcxErrorKind {
    // Common
    #[error("Object is in invalid state for requested operation")]
    InvalidState,
    #[error("Invalid Configuration")]
    InvalidConfiguration,
    #[error("Obj was not found with handle")]
    InvalidHandle,
    #[error("Invalid JSON string")]
    InvalidJson,
    #[error("Invalid Option")]
    InvalidOption,
    #[error("Invalid MessagePack")]
    InvalidMessagePack,
    #[error("Object cache error")]
    ObjectCacheError,
    #[error("Object not ready for specified action")]
    NotReady,
    #[error("IO Error, possibly creating a backup wallet")]
    IOError,
    #[error("Object (json, config, key, credential and etc...) passed to libindy has invalid structure")]
    LibindyInvalidStructure,
    #[error("Waiting for callback timed out")]
    TimeoutLibindy,
    #[error("Parameter passed to libindy was invalid")]
    InvalidLibindyParam,
    #[error("Library already initialized")]
    AlreadyInitialized,
    #[error("Action is not supported")]
    ActionNotSupported,
    #[error("Invalid input parameter")]
    InvalidInput,
    #[error("Unimplemented feature")]
    UnimplementedFeature,

    // Connection
    #[error("Could not create connection")]
    CreateConnection,
    #[error("Invalid Connection Handle")]
    InvalidConnectionHandle,
    #[error("Invalid invite details structure")]
    InvalidInviteDetail,
    #[error("Invalid redirect details structure")]
    InvalidRedirectDetail,
    #[error("Cannot Delete Connection. Check status of connection is appropriate to be deleted from agency.")]
    DeleteConnection,
    #[error("Error with Connection")]
    GeneralConnectionError,

    // Payment
    #[error("No payment information associated with object")]
    NoPaymentInformation,
    #[error("Insufficient amount of tokens to process request")]
    InsufficientTokenAmount,
    #[error("Invalid payment address")]
    InvalidPaymentAddress,

    // Credential Definition error
    #[error("Call to create Credential Definition failed")]
    CreateCredDef,
    #[error("Can't create, Credential Def already on ledger")]
    CredDefAlreadyCreated,
    #[error("Invalid Credential Definition handle")]
    InvalidCredDefHandle,
    #[error("No revocation delta found in storage for this revocation registry. Were any credentials locally revoked?")]
    RevDeltaNotFound,
    #[error("Failed to clean stored revocation delta")]
    RevDeltaFailedToClear,

    // Revocation
    #[error("Failed to create Revocation Registration Definition")]
    CreateRevRegDef,
    #[error("Invalid Revocation Details")]
    InvalidRevocationDetails,
    #[error("Unable to Update Revocation Delta On Ledger")]
    InvalidRevocationEntry,
    #[error("Invalid Credential Revocation timestamp")]
    InvalidRevocationTimestamp,
    #[error("No revocation definition found")]
    RevRegDefNotFound,

    // Credential
    #[error("Invalid credential handle")]
    InvalidCredentialHandle,
    #[error("could not create credential request")]
    CreateCredentialRequest,

    // Issuer Credential
    #[error("Invalid Credential Issuer Handle")]
    InvalidIssuerCredentialHandle,
    #[error("Invalid Credential Request")]
    InvalidCredentialRequest,
    #[error("Invalid credential json")]
    InvalidCredential,
    #[error("Attributes provided to Credential Offer are not correct, possibly malformed")]
    InvalidAttributesStructure,

    // Proof
    #[error("Invalid proof handle")]
    InvalidProofHandle,
    #[error("Obj was not found with handle")]
    InvalidDisclosedProofHandle,
    #[error("Proof had invalid format")]
    InvalidProof,
    #[error("Schema was invalid or corrupt")]
    InvalidSchema,
    #[error("The Proof received does not have valid credentials listed.")]
    InvalidProofCredentialData,
    #[error("Could not create proof")]
    CreateProof,
    #[error("Proof Request Passed into Libindy Call Was Invalid")]
    InvalidProofRequest,

    // Schema
    #[error("Could not create schema")]
    CreateSchema,
    #[error("Invalid Schema Handle")]
    InvalidSchemaHandle,
    #[error("No Schema for that schema sequence number")]
    InvalidSchemaSeqNo,
    #[error("Duplicate Schema: Ledger Already Contains Schema For Given DID, Version, and Name Combination")]
    DuplicationSchema,
    #[error("Unknown Rejection of Schema Creation, refer to libindy documentation")]
    UnknownSchemaRejection,

    // Public agent
    #[error("Could not create public agent")]
    CreatePublicAgent,

    // Out of Band
    #[error("Could not create out of band message.")]
    CreateOutOfBand,

    // Pool
    #[error("Invalid genesis transactions path.")]
    InvalidGenesisTxnPath,
    #[error("Formatting for Pool Config are incorrect.")]
    CreatePoolConfig,
    #[error("Connection to Pool Ledger.")]
    PoolLedgerConnect,
    #[error("Ledger rejected submitted request.")]
    InvalidLedgerResponse,
    #[error("No Pool open. Can't return handle.")]
    NoPoolOpen,
    #[error("Message failed in post")]
    PostMessageFailed,

    // Wallet
    #[error("Error Creating a wallet")]
    WalletCreate,
    #[error("Missing wallet name in config")]
    MissingWalletName,
    #[error("Missing exported wallet path in config")]
    MissingExportedWalletPath,
    #[error("Missing exported backup key in config")]
    MissingBackupKey,
    #[error("Attempt to open wallet with invalid credentials")]
    WalletAccessFailed,
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

    // Logger
    #[error("Logging Error")]
    LoggingError,

    // Validation
    #[error("Could not encode string to a big integer.")]
    EncodeError,
    #[error("Unknown Error")]
    UnknownError,
    #[error("Invalid DID")]
    InvalidDid,
    #[error("Invalid VERKEY")]
    InvalidVerkey,
    #[error("Invalid NONCE")]
    InvalidNonce,
    #[error("Invalid URL")]
    InvalidUrl,
    #[error("Configuration is missing the Payment Method parameter")]
    MissingPaymentMethod,
    #[error("Unable to serialize")]
    SerializationError,
    #[error("Value needs to be base58")]
    NotBase58,
    #[error("Could not parse a value")]
    ParsingError,

    // A2A
    #[error("Invalid HTTP response.")]
    InvalidHttpResponse,
    #[error("No Endpoint set for Connection Object")]
    NoEndpoint,
    #[error("Error Retrieving messages from API")]
    InvalidMessages,
    #[error("Error creating agent in agency")]
    CreateAgent,

    #[error("Common error {}", 0)]
    Common(u32),
    #[error("Libndy error {}", 0)]
    LibndyError(u32),
    #[error("Unknown libindy error")]
    UnknownLibndyError,
    #[error("No Agent pairwise information")]
    NoAgentInformation,

    #[error("Invalid message format")]
    InvalidMessageFormat,

    #[error("Attempted to unlock poisoned lock")]
    PoisonedLock,
}

#[derive(Debug, thiserror::Error)]
pub struct VcxError {
    msg: String,
    kind: VcxErrorKind,
}

impl fmt::Display for VcxError {
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

impl VcxError {
    pub fn from_msg<D>(kind: VcxErrorKind, msg: D) -> VcxError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        VcxError {
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

    pub fn kind(&self) -> VcxErrorKind {
        self.kind
    }

    pub fn extend<D>(self, msg: D) -> VcxError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        VcxError {
            msg: msg.to_string(),
            ..self
        }
    }

    pub fn map<D>(self, kind: VcxErrorKind, msg: D) -> VcxError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        VcxError {
            msg: msg.to_string(),
            kind,
            ..self
        }
    }
}

pub fn err_msg<D>(kind: VcxErrorKind, msg: D) -> VcxError
where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    VcxError::from_msg(kind, msg)
}

impl From<VcxErrorKind> for VcxError {
    fn from(kind: VcxErrorKind) -> VcxError {
        VcxError::from_msg(kind, crate::utils::error::error_message(&kind.into()))
    }
}

impl From<agency_client::error::AgencyClientError> for VcxError {
    fn from(agency_err: agency_client::error::AgencyClientError) -> VcxError {
        let vcx_error_kind: VcxErrorKind = agency_err.kind().into();
        VcxError::from_msg(vcx_error_kind, agency_err.to_string())
    }
}

impl From<SenderConfigBuilderError> for VcxError {
    fn from(err: SenderConfigBuilderError) -> VcxError {
        let vcx_error_kind = VcxErrorKind::InvalidConfiguration;
        VcxError::from_msg(vcx_error_kind, err.to_string())
    }
}

impl From<AgencyClientErrorKind> for VcxErrorKind {
    fn from(agency_err: AgencyClientErrorKind) -> VcxErrorKind {
        match agency_err {
            AgencyClientErrorKind::InvalidState => VcxErrorKind::InvalidState,
            AgencyClientErrorKind::InvalidConfiguration => VcxErrorKind::InvalidConfiguration,
            AgencyClientErrorKind::InvalidJson => VcxErrorKind::InvalidJson,
            AgencyClientErrorKind::InvalidOption => VcxErrorKind::InvalidOption,
            AgencyClientErrorKind::InvalidMessagePack => VcxErrorKind::InvalidMessagePack,
            AgencyClientErrorKind::IOError => VcxErrorKind::IOError,
            AgencyClientErrorKind::LibindyInvalidStructure => VcxErrorKind::LibindyInvalidStructure,
            AgencyClientErrorKind::TimeoutLibindy => VcxErrorKind::TimeoutLibindy,
            AgencyClientErrorKind::InvalidLibindyParam => VcxErrorKind::InvalidLibindyParam,
            AgencyClientErrorKind::PostMessageFailed => VcxErrorKind::PostMessageFailed,
            AgencyClientErrorKind::InvalidWalletHandle => VcxErrorKind::InvalidWalletHandle,
            AgencyClientErrorKind::DuplicationWallet => VcxErrorKind::DuplicationWallet,
            AgencyClientErrorKind::WalletRecordNotFound => VcxErrorKind::WalletRecordNotFound,
            AgencyClientErrorKind::DuplicationWalletRecord => VcxErrorKind::DuplicationWalletRecord,
            AgencyClientErrorKind::WalletNotFound => VcxErrorKind::WalletNotFound,
            AgencyClientErrorKind::WalletAlreadyOpen => VcxErrorKind::WalletAlreadyOpen,
            AgencyClientErrorKind::MissingWalletKey => VcxErrorKind::MissingWalletKey,
            AgencyClientErrorKind::DuplicationMasterSecret => VcxErrorKind::DuplicationMasterSecret,
            AgencyClientErrorKind::DuplicationDid => VcxErrorKind::DuplicationDid,
            AgencyClientErrorKind::UnknownError => VcxErrorKind::UnknownError,
            AgencyClientErrorKind::InvalidDid => VcxErrorKind::InvalidDid,
            AgencyClientErrorKind::InvalidVerkey => VcxErrorKind::InvalidVerkey,
            AgencyClientErrorKind::InvalidUrl => VcxErrorKind::InvalidUrl,
            AgencyClientErrorKind::SerializationError => VcxErrorKind::SerializationError,
            AgencyClientErrorKind::NotBase58 => VcxErrorKind::NotBase58,
            AgencyClientErrorKind::InvalidHttpResponse => VcxErrorKind::InvalidHttpResponse,
            AgencyClientErrorKind::CreateAgent => VcxErrorKind::CreateAgent,
            AgencyClientErrorKind::LibndyError(v) => VcxErrorKind::LibndyError(v),
            AgencyClientErrorKind::UnknownLibndyError => VcxErrorKind::UnknownLibndyError,
        }
    }
}

impl From<messages::error::MessagesError> for VcxError {
    fn from(msg_err: messages::error::MessagesError) -> VcxError {
        let vcx_error_kind: VcxErrorKind = msg_err.kind().into();
        VcxError::from_msg(vcx_error_kind, msg_err.to_string())
    }
}

impl From<MessagesErrorKind> for VcxErrorKind {
    fn from(msg_err: MessagesErrorKind) -> VcxErrorKind {
        match msg_err {
            MessagesErrorKind::InvalidState => VcxErrorKind::InvalidState,
            MessagesErrorKind::InvalidConfiguration => VcxErrorKind::InvalidConfiguration,
            MessagesErrorKind::InvalidJson => VcxErrorKind::InvalidJson,
            MessagesErrorKind::InvalidOption => VcxErrorKind::InvalidOption,
            MessagesErrorKind::InvalidMessagePack => VcxErrorKind::InvalidMessagePack,
            MessagesErrorKind::IOError => VcxErrorKind::IOError,
            MessagesErrorKind::LibindyInvalidStructure => VcxErrorKind::LibindyInvalidStructure,
            MessagesErrorKind::TimeoutLibindy => VcxErrorKind::TimeoutLibindy,
            MessagesErrorKind::InvalidLibindyParam => VcxErrorKind::InvalidLibindyParam,
            MessagesErrorKind::PostMessageFailed => VcxErrorKind::PostMessageFailed,
            MessagesErrorKind::InvalidWalletHandle => VcxErrorKind::InvalidWalletHandle,
            MessagesErrorKind::DuplicationWallet => VcxErrorKind::DuplicationWallet,
            MessagesErrorKind::WalletRecordNotFound => VcxErrorKind::WalletRecordNotFound,
            MessagesErrorKind::DuplicationWalletRecord => VcxErrorKind::DuplicationWalletRecord,
            MessagesErrorKind::WalletNotFound => VcxErrorKind::WalletNotFound,
            MessagesErrorKind::WalletAlreadyOpen => VcxErrorKind::WalletAlreadyOpen,
            MessagesErrorKind::MissingWalletKey => VcxErrorKind::MissingWalletKey,
            MessagesErrorKind::DuplicationMasterSecret => VcxErrorKind::DuplicationMasterSecret,
            MessagesErrorKind::DuplicationDid => VcxErrorKind::DuplicationDid,
            MessagesErrorKind::UnknownError => VcxErrorKind::UnknownError,
            MessagesErrorKind::InvalidDid => VcxErrorKind::InvalidDid,
            MessagesErrorKind::InvalidVerkey => VcxErrorKind::InvalidVerkey,
            MessagesErrorKind::InvalidUrl => VcxErrorKind::InvalidUrl,
            MessagesErrorKind::SerializationError => VcxErrorKind::SerializationError,
            MessagesErrorKind::NotBase58 => VcxErrorKind::NotBase58,
            MessagesErrorKind::InvalidHttpResponse => VcxErrorKind::InvalidHttpResponse,
            MessagesErrorKind::CreateAgent => VcxErrorKind::CreateAgent,
            MessagesErrorKind::LibndyError(v) => VcxErrorKind::LibndyError(v),
            _ => VcxErrorKind::UnknownLibndyError,
        }
    }
}


impl<T> From<sync::PoisonError<T>> for VcxError {
    fn from(err: sync::PoisonError<T>) -> Self {
        VcxError::from_msg(VcxErrorKind::PoisonedLock, err.to_string())
    }
}

pub type VcxResult<T> = Result<T, VcxError>;

/// Extension methods for `Result`.
impl From<VcxError> for u32 {
    fn from(code: VcxError) -> u32 {
        code.kind().into()
    }
}

impl From<VcxErrorKind> for u32 {
    fn from(code: VcxErrorKind) -> u32 {
        match code {
            VcxErrorKind::InvalidState => error::INVALID_STATE.code_num,
            VcxErrorKind::InvalidConfiguration => error::INVALID_CONFIGURATION.code_num,
            VcxErrorKind::InvalidHandle => error::INVALID_OBJ_HANDLE.code_num,
            VcxErrorKind::InvalidJson => error::INVALID_JSON.code_num,
            VcxErrorKind::InvalidOption => error::INVALID_OPTION.code_num,
            VcxErrorKind::InvalidMessagePack => error::INVALID_MSGPACK.code_num,
            VcxErrorKind::ObjectCacheError => error::OBJECT_CACHE_ERROR.code_num,
            VcxErrorKind::NoPaymentInformation => error::NO_PAYMENT_INFORMATION.code_num,
            VcxErrorKind::NotReady => error::NOT_READY.code_num,
            VcxErrorKind::InvalidRevocationDetails => error::INVALID_REVOCATION_DETAILS.code_num,
            VcxErrorKind::GeneralConnectionError => error::CONNECTION_ERROR.code_num,
            VcxErrorKind::IOError => error::IOERROR.code_num,
            VcxErrorKind::LibindyInvalidStructure => error::LIBINDY_INVALID_STRUCTURE.code_num,
            VcxErrorKind::TimeoutLibindy => error::TIMEOUT_LIBINDY_ERROR.code_num,
            VcxErrorKind::InvalidLibindyParam => error::INVALID_LIBINDY_PARAM.code_num,
            VcxErrorKind::AlreadyInitialized => error::ALREADY_INITIALIZED.code_num,
            VcxErrorKind::CreateConnection => error::CREATE_CONNECTION_ERROR.code_num,
            VcxErrorKind::InvalidConnectionHandle => error::INVALID_CONNECTION_HANDLE.code_num,
            VcxErrorKind::InvalidInviteDetail => error::INVALID_INVITE_DETAILS.code_num,
            VcxErrorKind::InvalidRedirectDetail => error::INVALID_REDIRECT_DETAILS.code_num,
            VcxErrorKind::DeleteConnection => error::CANNOT_DELETE_CONNECTION.code_num,
            VcxErrorKind::CreateCredDef => error::CREATE_CREDENTIAL_DEF_ERR.code_num,
            VcxErrorKind::CredDefAlreadyCreated => error::CREDENTIAL_DEF_ALREADY_CREATED.code_num,
            VcxErrorKind::InvalidCredDefHandle => error::INVALID_CREDENTIAL_DEF_HANDLE.code_num,
            VcxErrorKind::InvalidRevocationEntry => error::INVALID_REV_ENTRY.code_num,
            VcxErrorKind::CreateRevRegDef => error::INVALID_REV_REG_DEF_CREATION.code_num,
            VcxErrorKind::InvalidCredentialHandle => error::INVALID_CREDENTIAL_HANDLE.code_num,
            VcxErrorKind::CreateCredentialRequest => error::CREATE_CREDENTIAL_REQUEST_ERROR.code_num,
            VcxErrorKind::InvalidIssuerCredentialHandle => error::INVALID_ISSUER_CREDENTIAL_HANDLE.code_num,
            VcxErrorKind::InvalidCredentialRequest => error::INVALID_CREDENTIAL_REQUEST.code_num,
            VcxErrorKind::InvalidCredential => error::INVALID_CREDENTIAL_JSON.code_num,
            VcxErrorKind::InsufficientTokenAmount => error::INSUFFICIENT_TOKEN_AMOUNT.code_num,
            VcxErrorKind::InvalidProofHandle => error::INVALID_PROOF_HANDLE.code_num,
            VcxErrorKind::InvalidDisclosedProofHandle => error::INVALID_DISCLOSED_PROOF_HANDLE.code_num,
            VcxErrorKind::InvalidProof => error::INVALID_PROOF.code_num,
            VcxErrorKind::InvalidSchema => error::INVALID_SCHEMA.code_num,
            VcxErrorKind::InvalidProofCredentialData => error::INVALID_PROOF_CREDENTIAL_DATA.code_num,
            VcxErrorKind::CreateProof => error::CREATE_PROOF_ERROR.code_num,
            VcxErrorKind::InvalidRevocationTimestamp => error::INVALID_REVOCATION_TIMESTAMP.code_num,
            VcxErrorKind::CreateSchema => error::INVALID_SCHEMA_CREATION.code_num,
            VcxErrorKind::InvalidSchemaHandle => error::INVALID_SCHEMA_HANDLE.code_num,
            VcxErrorKind::InvalidSchemaSeqNo => error::INVALID_SCHEMA_SEQ_NO.code_num,
            VcxErrorKind::DuplicationSchema => error::DUPLICATE_SCHEMA.code_num,
            VcxErrorKind::UnknownSchemaRejection => error::UNKNOWN_SCHEMA_REJECTION.code_num,
            VcxErrorKind::WalletCreate => error::INVALID_WALLET_CREATION.code_num,
            VcxErrorKind::MissingWalletName => error::MISSING_WALLET_NAME.code_num,
            VcxErrorKind::WalletAccessFailed => error::WALLET_ACCESS_FAILED.code_num,
            VcxErrorKind::InvalidWalletHandle => error::INVALID_WALLET_HANDLE.code_num,
            VcxErrorKind::DuplicationWallet => error::WALLET_ALREADY_EXISTS.code_num,
            VcxErrorKind::WalletNotFound => error::WALLET_NOT_FOUND.code_num,
            VcxErrorKind::WalletRecordNotFound => error::WALLET_RECORD_NOT_FOUND.code_num,
            VcxErrorKind::PoolLedgerConnect => error::POOL_LEDGER_CONNECT.code_num,
            VcxErrorKind::InvalidGenesisTxnPath => error::INVALID_GENESIS_TXN_PATH.code_num,
            VcxErrorKind::CreatePoolConfig => error::CREATE_POOL_CONFIG.code_num,
            VcxErrorKind::DuplicationWalletRecord => error::DUPLICATE_WALLET_RECORD.code_num,
            VcxErrorKind::WalletAlreadyOpen => error::WALLET_ALREADY_OPEN.code_num,
            VcxErrorKind::DuplicationMasterSecret => error::DUPLICATE_MASTER_SECRET.code_num,
            VcxErrorKind::DuplicationDid => error::DID_ALREADY_EXISTS_IN_WALLET.code_num,
            VcxErrorKind::InvalidLedgerResponse => error::INVALID_LEDGER_RESPONSE.code_num,
            VcxErrorKind::InvalidAttributesStructure => error::INVALID_ATTRIBUTES_STRUCTURE.code_num,
            VcxErrorKind::InvalidPaymentAddress => error::INVALID_PAYMENT_ADDRESS.code_num,
            VcxErrorKind::NoEndpoint => error::NO_ENDPOINT.code_num,
            VcxErrorKind::InvalidProofRequest => error::INVALID_PROOF_REQUEST.code_num,
            VcxErrorKind::NoPoolOpen => error::NO_POOL_OPEN.code_num,
            VcxErrorKind::PostMessageFailed => error::POST_MSG_FAILURE.code_num,
            VcxErrorKind::LoggingError => error::LOGGING_ERROR.code_num,
            VcxErrorKind::EncodeError => error::BIG_NUMBER_ERROR.code_num,
            VcxErrorKind::UnknownError => error::UNKNOWN_ERROR.code_num,
            VcxErrorKind::InvalidDid => error::INVALID_DID.code_num,
            VcxErrorKind::InvalidVerkey => error::INVALID_VERKEY.code_num,
            VcxErrorKind::InvalidNonce => error::INVALID_NONCE.code_num,
            VcxErrorKind::InvalidUrl => error::INVALID_URL.code_num,
            VcxErrorKind::MissingWalletKey => error::MISSING_WALLET_KEY.code_num,
            VcxErrorKind::MissingPaymentMethod => error::MISSING_PAYMENT_METHOD.code_num,
            VcxErrorKind::SerializationError => error::SERIALIZATION_ERROR.code_num,
            VcxErrorKind::NotBase58 => error::NOT_BASE58.code_num,
            VcxErrorKind::InvalidHttpResponse => error::INVALID_HTTP_RESPONSE.code_num,
            VcxErrorKind::InvalidMessages => error::INVALID_MESSAGES.code_num,
            VcxErrorKind::MissingExportedWalletPath => error::MISSING_EXPORTED_WALLET_PATH.code_num,
            VcxErrorKind::MissingBackupKey => error::MISSING_BACKUP_KEY.code_num,
            VcxErrorKind::UnknownLibndyError => error::UNKNOWN_LIBINDY_ERROR.code_num,
            VcxErrorKind::ActionNotSupported => error::ACTION_NOT_SUPPORTED.code_num,
            VcxErrorKind::Common(num) => num,
            VcxErrorKind::LibndyError(num) => num,
            VcxErrorKind::NoAgentInformation => error::NO_AGENT_INFO.code_num,
            VcxErrorKind::RevRegDefNotFound => error::REV_REG_DEF_NOT_FOUND.code_num,
            VcxErrorKind::RevDeltaNotFound => error::REV_DELTA_NOT_FOUND.code_num,
            VcxErrorKind::RevDeltaFailedToClear => error::REV_DELTA_FAILED_TO_CLEAR.code_num,
            VcxErrorKind::PoisonedLock => error::POISONED_LOCK.code_num,
            VcxErrorKind::InvalidMessageFormat => error::INVALID_MESSAGE_FORMAT.code_num,
            VcxErrorKind::CreatePublicAgent => error::CREATE_PUBLIC_AGENT.code_num,
            VcxErrorKind::CreateOutOfBand => error::CREATE_OUT_OF_BAND.code_num,
            VcxErrorKind::CreateAgent => error::CREATE_AGENT.code_num,
            VcxErrorKind::InvalidInput => error::INVALID_INPUT.code_num,
            VcxErrorKind::ParsingError => error::PARSING.code_num,
            VcxErrorKind::UnimplementedFeature => error::UNIMPLEMENTED_FEATURE.code_num,
        }
    }
}

impl From<u32> for VcxErrorKind {
    fn from(code: u32) -> VcxErrorKind {
        match code {
            _ if { error::INVALID_STATE.code_num == code } => VcxErrorKind::InvalidState,
            _ if { error::INVALID_CONFIGURATION.code_num == code } => VcxErrorKind::InvalidConfiguration,
            _ if { error::INVALID_OBJ_HANDLE.code_num == code } => VcxErrorKind::InvalidHandle,
            _ if { error::INVALID_JSON.code_num == code } => VcxErrorKind::InvalidJson,
            _ if { error::INVALID_OPTION.code_num == code } => VcxErrorKind::InvalidOption,
            _ if { error::INVALID_MSGPACK.code_num == code } => VcxErrorKind::InvalidMessagePack,
            _ if { error::OBJECT_CACHE_ERROR.code_num == code } => VcxErrorKind::ObjectCacheError,
            _ if { error::NO_PAYMENT_INFORMATION.code_num == code } => VcxErrorKind::NoPaymentInformation,
            _ if { error::NOT_READY.code_num == code } => VcxErrorKind::NotReady,
            _ if { error::INVALID_REVOCATION_DETAILS.code_num == code } => VcxErrorKind::InvalidRevocationDetails,
            _ if { error::CONNECTION_ERROR.code_num == code } => VcxErrorKind::GeneralConnectionError,
            _ if { error::IOERROR.code_num == code } => VcxErrorKind::IOError,
            _ if { error::LIBINDY_INVALID_STRUCTURE.code_num == code } => VcxErrorKind::LibindyInvalidStructure,
            _ if { error::TIMEOUT_LIBINDY_ERROR.code_num == code } => VcxErrorKind::TimeoutLibindy,
            _ if { error::INVALID_LIBINDY_PARAM.code_num == code } => VcxErrorKind::InvalidLibindyParam,
            _ if { error::ALREADY_INITIALIZED.code_num == code } => VcxErrorKind::AlreadyInitialized,
            _ if { error::CREATE_CONNECTION_ERROR.code_num == code } => VcxErrorKind::CreateConnection,
            _ if { error::INVALID_CONNECTION_HANDLE.code_num == code } => VcxErrorKind::InvalidConnectionHandle,
            _ if { error::INVALID_INVITE_DETAILS.code_num == code } => VcxErrorKind::InvalidInviteDetail,
            _ if { error::INVALID_REDIRECT_DETAILS.code_num == code } => VcxErrorKind::InvalidRedirectDetail,
            _ if { error::CANNOT_DELETE_CONNECTION.code_num == code } => VcxErrorKind::DeleteConnection,
            _ if { error::CREATE_CREDENTIAL_DEF_ERR.code_num == code } => VcxErrorKind::CreateCredDef,
            _ if { error::CREDENTIAL_DEF_ALREADY_CREATED.code_num == code } => VcxErrorKind::CredDefAlreadyCreated,
            _ if { error::INVALID_CREDENTIAL_DEF_HANDLE.code_num == code } => VcxErrorKind::InvalidCredDefHandle,
            _ if { error::INVALID_REV_ENTRY.code_num == code } => VcxErrorKind::InvalidRevocationEntry,
            _ if { error::INVALID_REV_REG_DEF_CREATION.code_num == code } => VcxErrorKind::CreateRevRegDef,
            _ if { error::INVALID_CREDENTIAL_HANDLE.code_num == code } => VcxErrorKind::InvalidCredentialHandle,
            _ if { error::CREATE_CREDENTIAL_REQUEST_ERROR.code_num == code } => VcxErrorKind::CreateCredentialRequest,
            _ if { error::INVALID_ISSUER_CREDENTIAL_HANDLE.code_num == code } => {
                VcxErrorKind::InvalidIssuerCredentialHandle
            }
            _ if { error::INVALID_CREDENTIAL_REQUEST.code_num == code } => VcxErrorKind::InvalidCredentialRequest,
            _ if { error::INVALID_CREDENTIAL_JSON.code_num == code } => VcxErrorKind::InvalidCredential,
            _ if { error::INSUFFICIENT_TOKEN_AMOUNT.code_num == code } => VcxErrorKind::InsufficientTokenAmount,
            _ if { error::INVALID_PROOF_HANDLE.code_num == code } => VcxErrorKind::InvalidProofHandle,
            _ if { error::INVALID_DISCLOSED_PROOF_HANDLE.code_num == code } => {
                VcxErrorKind::InvalidDisclosedProofHandle
            }
            _ if { error::INVALID_PROOF.code_num == code } => VcxErrorKind::InvalidProof,
            _ if { error::INVALID_SCHEMA.code_num == code } => VcxErrorKind::InvalidSchema,
            _ if { error::INVALID_PROOF_CREDENTIAL_DATA.code_num == code } => VcxErrorKind::InvalidProofCredentialData,
            _ if { error::CREATE_PROOF_ERROR.code_num == code } => VcxErrorKind::CreateProof,
            _ if { error::INVALID_REVOCATION_TIMESTAMP.code_num == code } => VcxErrorKind::InvalidRevocationTimestamp,
            _ if { error::INVALID_SCHEMA_CREATION.code_num == code } => VcxErrorKind::CreateSchema,
            _ if { error::INVALID_SCHEMA_HANDLE.code_num == code } => VcxErrorKind::InvalidSchemaHandle,
            _ if { error::INVALID_SCHEMA_SEQ_NO.code_num == code } => VcxErrorKind::InvalidSchemaSeqNo,
            _ if { error::DUPLICATE_SCHEMA.code_num == code } => VcxErrorKind::DuplicationSchema,
            _ if { error::UNKNOWN_SCHEMA_REJECTION.code_num == code } => VcxErrorKind::UnknownSchemaRejection,
            _ if { error::INVALID_WALLET_CREATION.code_num == code } => VcxErrorKind::WalletCreate,
            _ if { error::MISSING_WALLET_NAME.code_num == code } => VcxErrorKind::MissingWalletName,
            _ if { error::WALLET_ACCESS_FAILED.code_num == code } => VcxErrorKind::WalletAccessFailed,
            _ if { error::INVALID_WALLET_HANDLE.code_num == code } => VcxErrorKind::InvalidWalletHandle,
            _ if { error::WALLET_ALREADY_EXISTS.code_num == code } => VcxErrorKind::DuplicationWallet,
            _ if { error::WALLET_NOT_FOUND.code_num == code } => VcxErrorKind::WalletNotFound,
            _ if { error::WALLET_RECORD_NOT_FOUND.code_num == code } => VcxErrorKind::WalletRecordNotFound,
            _ if { error::POOL_LEDGER_CONNECT.code_num == code } => VcxErrorKind::PoolLedgerConnect,
            _ if { error::INVALID_GENESIS_TXN_PATH.code_num == code } => VcxErrorKind::InvalidGenesisTxnPath,
            _ if { error::CREATE_POOL_CONFIG.code_num == code } => VcxErrorKind::CreatePoolConfig,
            _ if { error::DUPLICATE_WALLET_RECORD.code_num == code } => VcxErrorKind::DuplicationWalletRecord,
            _ if { error::WALLET_ALREADY_OPEN.code_num == code } => VcxErrorKind::WalletAlreadyOpen,
            _ if { error::DUPLICATE_MASTER_SECRET.code_num == code } => VcxErrorKind::DuplicationMasterSecret,
            _ if { error::DID_ALREADY_EXISTS_IN_WALLET.code_num == code } => VcxErrorKind::DuplicationDid,
            _ if { error::INVALID_LEDGER_RESPONSE.code_num == code } => VcxErrorKind::InvalidLedgerResponse,
            _ if { error::INVALID_ATTRIBUTES_STRUCTURE.code_num == code } => VcxErrorKind::InvalidAttributesStructure,
            _ if { error::INVALID_PAYMENT_ADDRESS.code_num == code } => VcxErrorKind::InvalidPaymentAddress,
            _ if { error::NO_ENDPOINT.code_num == code } => VcxErrorKind::NoEndpoint,
            _ if { error::INVALID_PROOF_REQUEST.code_num == code } => VcxErrorKind::InvalidProofRequest,
            _ if { error::NO_POOL_OPEN.code_num == code } => VcxErrorKind::NoPoolOpen,
            _ if { error::POST_MSG_FAILURE.code_num == code } => VcxErrorKind::PostMessageFailed,
            _ if { error::LOGGING_ERROR.code_num == code } => VcxErrorKind::LoggingError,
            _ if { error::BIG_NUMBER_ERROR.code_num == code } => VcxErrorKind::EncodeError,
            _ if { error::UNKNOWN_ERROR.code_num == code } => VcxErrorKind::UnknownError,
            _ if { error::INVALID_DID.code_num == code } => VcxErrorKind::InvalidDid,
            _ if { error::INVALID_VERKEY.code_num == code } => VcxErrorKind::InvalidVerkey,
            _ if { error::INVALID_NONCE.code_num == code } => VcxErrorKind::InvalidNonce,
            _ if { error::INVALID_URL.code_num == code } => VcxErrorKind::InvalidUrl,
            _ if { error::MISSING_WALLET_KEY.code_num == code } => VcxErrorKind::MissingWalletKey,
            _ if { error::MISSING_PAYMENT_METHOD.code_num == code } => VcxErrorKind::MissingPaymentMethod,
            _ if { error::SERIALIZATION_ERROR.code_num == code } => VcxErrorKind::SerializationError,
            _ if { error::NOT_BASE58.code_num == code } => VcxErrorKind::NotBase58,
            _ if { error::INVALID_HTTP_RESPONSE.code_num == code } => VcxErrorKind::InvalidHttpResponse,
            _ if { error::INVALID_MESSAGES.code_num == code } => VcxErrorKind::InvalidMessages,
            _ if { error::MISSING_EXPORTED_WALLET_PATH.code_num == code } => VcxErrorKind::MissingExportedWalletPath,
            _ if { error::MISSING_BACKUP_KEY.code_num == code } => VcxErrorKind::MissingBackupKey,
            _ if { error::UNKNOWN_LIBINDY_ERROR.code_num == code } => VcxErrorKind::UnknownLibndyError,
            _ if { error::ACTION_NOT_SUPPORTED.code_num == code } => VcxErrorKind::ActionNotSupported,
            _ if { error::NO_AGENT_INFO.code_num == code } => VcxErrorKind::NoAgentInformation,
            _ if { error::REV_REG_DEF_NOT_FOUND.code_num == code } => VcxErrorKind::RevRegDefNotFound,
            _ if { error::REV_DELTA_NOT_FOUND.code_num == code } => VcxErrorKind::RevDeltaNotFound,
            _ if { error::CREATE_PUBLIC_AGENT.code_num == code } => VcxErrorKind::CreatePublicAgent,
            _ if { error::CREATE_OUT_OF_BAND.code_num == code } => VcxErrorKind::CreateOutOfBand,
            _ if { error::POISONED_LOCK.code_num == code } => VcxErrorKind::PoisonedLock,
            _ if { error::INVALID_MESSAGE_FORMAT.code_num == code } => VcxErrorKind::InvalidMessageFormat,
            _ if { error::CREATE_OUT_OF_BAND.code_num == code } => VcxErrorKind::CreateOutOfBand,
            _ if { error::CREATE_AGENT.code_num == code } => VcxErrorKind::CreateAgent,
            _ if { error::REV_DELTA_FAILED_TO_CLEAR.code_num == code } => VcxErrorKind::RevDeltaFailedToClear,
            _ => VcxErrorKind::UnknownError,
        }
    }
}

impl From<serde_json::Error> for VcxError {
    fn from(_err: serde_json::Error) -> Self {
        VcxErrorKind::InvalidJson.into()
    }
}

impl From<types::errors::IndyErrorKind> for VcxErrorKind {
    fn from(indy: types::errors::IndyErrorKind) -> Self {
        use types::errors::IndyErrorKind::*;



        match indy {
            // 100..=111, 115..=129
            InvalidParam(_) => VcxErrorKind::InvalidLibindyParam,

            // 112
            // InvalidState => VcxErrorKind::LibndyError(err_code),

            // 113
            InvalidStructure => VcxErrorKind::LibindyInvalidStructure,

            // 114
            IOError => VcxErrorKind::IOError,

            // 200
            InvalidWalletHandle => VcxErrorKind::InvalidWalletHandle,

            // 203
            WalletAlreadyExists => VcxErrorKind::DuplicationWallet,

            // 204
            WalletNotFound => VcxErrorKind::WalletNotFound,

            // 206
            WalletAlreadyOpened => VcxErrorKind::WalletAlreadyOpen,

            // 212
            WalletItemNotFound => VcxErrorKind::WalletRecordNotFound,

            // 213
            WalletItemAlreadyExists => VcxErrorKind::DuplicationWalletRecord,

            // 306
            PoolConfigAlreadyExists => VcxErrorKind::CreatePoolConfig,

            // 404
            MasterSecretDuplicateName => VcxErrorKind::DuplicationMasterSecret,

            // 407
            CredDefAlreadyExists => VcxErrorKind::CredDefAlreadyCreated,

            // 600
            DIDAlreadyExists => VcxErrorKind::DuplicationDid,

            // 702
            PaymentInsufficientFunds => VcxErrorKind::InsufficientTokenAmount,

            InvalidState |
            ProofRejected |
            RevocationRegistryFull |
            LedgerItemNotFound |
            InvalidPoolHandle |
            UnknownWalletStorageType |
            InvalidUserRevocId |
            CredentialRevoked |
            NoConsensus |
            InvalidTransaction |
            PoolNotCreated |
            PoolTerminated |
            PoolTimeout |
            PoolIncompatibleProtocolVersion |
            UnknownCrypto |
            WalletStorageTypeAlreadyRegistered |
            WalletAccessFailed |
            WalletEncodingError |
            WalletStorageError |
            WalletEncryptionError |
            WalletQueryError |
            UnknownPaymentMethodType |
            IncompatiblePaymentMethods |
            PaymentSourceDoesNotExist |
            PaymentOperationNotSupported |
            PaymentExtraFunds |
            TransactionNotAllowed |
            QueryAccountDoesNotExist |
            InvalidVDRHandle |
            InvalidVDRNamespace |
            IncompatibleLedger => {
                let err_code = types::ErrorCode::from(indy) as u32;
                VcxErrorKind::LibndyError(err_code)
            }
        }
    }
}

impl From<types::errors::IndyError> for VcxError {
    fn from(indy: types::errors::IndyError) -> Self {

        let vcx_kind: VcxErrorKind = indy.kind().into();

        vcx_kind.into()
    }
}
