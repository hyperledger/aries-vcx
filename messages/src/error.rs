use std::fmt;
use std::error::Error;

use crate::utils::error;

pub mod prelude {
    pub use super::{err_msg, MessagesError, MesssagesErrorKind, MessagesResult};
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, thiserror::Error)]
pub enum MesssagesErrorKind {
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
    #[error("Unable to serialize")]
    SerializationError,
    #[error("Value needs to be base58")]
    NotBase58,

    // A2A
    #[error("Invalid HTTP response.")]
    InvalidHttpResponse,
    #[error("No Endpoint set for Connection Object")]
    NoEndpoint,
    #[error("Error Retrieving messages from API")]
    InvalidMessages,
    #[error("Error creating agent in agency")]
    CreateAgent,

    #[error("Unknown libindy error")]
    UnknownLibndyError,
    #[error("No Agent pairwise information")]
    NoAgentInformation,

    #[error("Invalid message format")]
    InvalidMessageFormat,
}

#[derive(Debug, thiserror::Error)]
pub struct MessagesError {
    msg: String,
    kind: MesssagesErrorKind,
}

impl fmt::Display for MessagesError {
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

impl MessagesError {
    pub fn from_msg<D>(kind: MesssagesErrorKind, msg: D) -> MessagesError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        MessagesError {
            msg: msg.to_string(),
            kind,
        }
    }

    pub fn kind(&self) -> MesssagesErrorKind {
        self.kind
    }
}

pub fn err_msg<D>(kind: MesssagesErrorKind, msg: D) -> MessagesError
where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    MessagesError::from_msg(kind, msg)
}
//
// impl From<MesssagesErrorKind> for MessagesError {
//     fn from(kind: MesssagesErrorKind) -> MessagesError {
//         MessagesError::from_msg(kind, crate::utils::error::error_message(&kind.into()))
//     }
// }

pub type MessagesResult<T> = Result<T, MessagesError>;
