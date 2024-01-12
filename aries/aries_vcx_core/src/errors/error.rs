use std::{error::Error, fmt};

use thiserror;

pub mod prelude {
    pub use super::{err_msg, AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, thiserror::Error)]
pub enum AriesVcxCoreErrorKind {
    // Common
    #[error("Object is in invalid state for requested operation")]
    InvalidState,
    #[error("Invalid Configuration")]
    InvalidConfiguration,
    #[error("Invalid JSON string")]
    InvalidJson,
    #[error("Invalid Option")]
    InvalidOption,
    #[error("Invalid MessagePack")]
    InvalidMessagePack,
    #[error("Object not ready for specified action")]
    NotReady,
    #[error("IO Error, possibly creating a backup wallet")]
    IOError,
    #[error(
        "Object (json, config, key, credential and etc...) passed to libindy has invalid structure"
    )]
    LibindyInvalidStructure,
    #[error("Parameter passed to libindy was invalid")]
    InvalidLibindyParam,
    #[error("Action is not supported")]
    ActionNotSupported,
    #[error("Invalid input parameter")]
    InvalidInput,
    #[error("Unimplemented feature")]
    UnimplementedFeature,

    // Credential Definition error
    #[error("Can't create, Credential Def already on ledger")]
    CredDefAlreadyCreated,
    #[error(
        "No revocation delta found in storage for this revocation registry. Were any credentials \
         locally revoked?"
    )]
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

    // Issuer Credential
    #[error("Attributes provided to Credential Offer are not correct, possibly malformed")]
    InvalidAttributesStructure,

    // Proof
    #[error("Proof had invalid format")]
    InvalidProof,
    #[error("Schema was invalid or corrupt")]
    InvalidSchema,
    #[error("The Proof received does not have valid credentials listed.")]
    InvalidProofCredentialData,
    #[error("Proof Request Passed into Libindy Call Was Invalid")]
    InvalidProofRequest,
    #[error("The proof was rejected")]
    ProofRejected,

    // Schema
    #[error("No Schema for that schema sequence number")]
    InvalidSchemaSeqNo,
    #[error(
        "Duplicate Schema: Ledger Already Contains Schema For Given DID, Version, and Name \
         Combination"
    )]
    DuplicationSchema,
    #[error("Unknown Rejection of Schema Creation, refer to libindy documentation")]
    UnknownSchemaRejection,

    // Pool
    #[error("Invalid genesis transactions path.")]
    InvalidGenesisTxnPath,
    #[error("Formatting for Pool Config are incorrect.")]
    CreatePoolConfig,
    #[error("Connection to Pool Ledger.")]
    PoolLedgerConnect,
    #[error("Ledger rejected submitted request.")]
    InvalidLedgerResponse,
    #[error("Ledger item not found.")]
    LedgerItemNotFound,
    #[error("No Pool open. Can't return handle.")]
    NoPoolOpen,
    #[error("Message failed in post")]
    PostMessageFailed,

    // Wallet
    #[error("Error Creating a wallet")]
    WalletCreate,
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
    #[error("Attempted to add a Master Secret that already existed in wallet")]
    DuplicationMasterSecret,
    #[error("Attempted to add a DID to wallet when that DID already exists in wallet")]
    DuplicationDid,

    #[error("Unexpected wallet error")]
    WalletError,

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
    #[error("Could not parse a value")]
    ParsingError,

    // A2A
    #[error("Invalid HTTP response.")]
    InvalidHttpResponse,
    #[error("Error Retrieving messages from API")]
    InvalidMessages,

    #[error("Libndy error {}", 0)]
    VdrToolsError(u32),
    #[error("Ursa error")]
    UrsaError,
    #[error("No Agent pairwise information")]
    NoAgentInformation,

    #[error("Invalid message format")]
    InvalidMessageFormat,
}

#[derive(Debug, thiserror::Error)]
pub struct AriesVcxCoreError {
    msg: String,
    kind: AriesVcxCoreErrorKind,
}

impl fmt::Display for AriesVcxCoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Error: {}\n", self.msg)?;
        let mut current = self.source();
        while let Some(cause) = current {
            writeln!(f, "Caused by:\n\t{cause}")?;
            current = cause.source();
        }
        Ok(())
    }
}

impl AriesVcxCoreError {
    pub fn from_msg<D>(kind: AriesVcxCoreErrorKind, msg: D) -> AriesVcxCoreError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        AriesVcxCoreError {
            msg: msg.to_string(),
            kind,
        }
    }

    pub fn find_root_cause(&self) -> String {
        let mut current = self.source();
        while let Some(cause) = current {
            if cause.source().is_none() {
                return cause.to_string();
            }
            current = cause.source();
        }
        self.to_string()
    }

    pub fn kind(&self) -> AriesVcxCoreErrorKind {
        self.kind
    }

    pub fn extend<D>(self, msg: D) -> AriesVcxCoreError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        AriesVcxCoreError {
            msg: msg.to_string(),
            ..self
        }
    }

    pub fn map<D>(self, kind: AriesVcxCoreErrorKind, msg: D) -> AriesVcxCoreError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        AriesVcxCoreError {
            msg: msg.to_string(),
            kind,
        }
    }
}

pub fn err_msg<D>(kind: AriesVcxCoreErrorKind, msg: D) -> AriesVcxCoreError
where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    AriesVcxCoreError::from_msg(kind, msg)
}

pub type VcxCoreResult<T> = Result<T, AriesVcxCoreError>;
