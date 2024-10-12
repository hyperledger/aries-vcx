use std::{error::Error, fmt};

use thiserror;

pub mod prelude {
    pub use super::{err_msg, AriesVcxError, AriesVcxErrorKind, VcxResult};
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, thiserror::Error)]
pub enum AriesVcxErrorKind {
    // Common
    #[error("Object is in invalid state for requested operation")]
    InvalidState,
    #[error("Invalid Configuration")]
    InvalidConfiguration,
    #[error("Authentication error")]
    AuthenticationError,
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

    #[error("Unexpected wallet error")]
    WalletError,

    // A2A
    #[error("Invalid HTTP response.")]
    InvalidHttpResponse,
    #[error("Error Retrieving messages from API")]
    InvalidMessages,

    #[error("Ursa error")]
    UrsaError,
    #[error("No Agent pairwise information")]
    NoAgentInformation,

    #[error("Invalid message format")]
    InvalidMessageFormat,
}

#[derive(thiserror::Error)]
pub struct AriesVcxError {
    msg: String,
    kind: AriesVcxErrorKind,
    backtrace: Option<String>,
}

fn format_error(err: &AriesVcxError, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "Error: {}", err.msg())?;
    match err.backtrace() {
        None => {}
        Some(backtrace) => {
            writeln!(f, "Backtrace: {}", backtrace)?;
        }
    }
    let mut current = err.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

impl fmt::Display for AriesVcxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format_error(self, f)
    }
}

impl fmt::Debug for AriesVcxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format_error(self, f)
    }
}

fn try_capture_backtrace() -> Option<String> {
    #[cfg(feature = "backtrace_errors")]
    {
        use backtrace::Backtrace;

        let backtrace = Backtrace::new();
        let mut filtered_backtrace = String::new();

        for frame in backtrace.frames() {
            let symbols = frame.symbols();
            if !symbols.is_empty() {
                let symbol = &symbols[0];
                if let Some(filename) = symbol.filename() {
                    if let Some(line) = symbol.lineno() {
                        filtered_backtrace.push_str(&format!("[{}:{}]", filename.display(), line));
                    }
                }
                if let Some(name) = symbol.name() {
                    filtered_backtrace.push_str(&format!(" {}", name));
                }
                filtered_backtrace.push('\n');
            }
        }
        Some(filtered_backtrace)
    }
    #[cfg(not(feature = "backtrace_errors"))]
    None
}

impl AriesVcxError {
    fn new(kind: AriesVcxErrorKind, msg: String) -> Self {
        AriesVcxError {
            msg,
            kind,
            backtrace: try_capture_backtrace(),
        }
    }

    pub fn from_msg<D>(kind: AriesVcxErrorKind, msg: D) -> AriesVcxError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        Self::new(kind, msg.to_string())
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

    pub fn kind(&self) -> AriesVcxErrorKind {
        self.kind
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    pub fn backtrace(&self) -> Option<&String> {
        self.backtrace.as_ref()
    }

    pub fn extend<D>(self, msg: D) -> AriesVcxError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        Self::new(self.kind, format!("{}\n{}", self.msg, msg))
    }

    pub fn map<D>(self, kind: AriesVcxErrorKind, msg: D) -> AriesVcxError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        Self::new(kind, msg.to_string())
    }
}

pub fn err_msg<D>(kind: AriesVcxErrorKind, msg: D) -> AriesVcxError
where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    AriesVcxError::from_msg(kind, msg)
}

pub type VcxResult<T> = Result<T, AriesVcxError>;
