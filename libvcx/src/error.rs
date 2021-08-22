use std::fmt;
use std::sync;

use failure::{Backtrace, Context, Fail};

use aries_vcx::utils;

pub mod prelude {
    pub use super::{err_msg, VcxError, VcxErrorExt, VcxErrorKind, VcxResult, VcxResultExt};
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

    #[fail(display = "Attempted to unlock poisoned lock")]
    PoisonedLock,

    #[fail(display = "Error creating public agent")]
    CreatePublicAgent,
}

#[derive(Debug)]
pub struct VcxError {
    inner: Context<VcxErrorKind>,
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
        VcxError::from_msg(kind, aries_vcx::utils::error::error_message(&kind.clone().into()))
    }
}

impl From<aries_vcx::agency_client::error::AgencyClientError> for VcxError {
    fn from(agency_err: aries_vcx::agency_client::error::AgencyClientError) -> VcxError {
        let kind_num: u32 = agency_err.kind().into();
        VcxError::from_msg(kind_num.into(), utils::error::error_message(&agency_err.kind().clone().into()))
    }
}

impl From<aries_vcx::error::VcxError> for VcxError {
    fn from(aries_err: aries_vcx::error::VcxError) -> VcxError {
        let kind_num: u32 = aries_err.kind().into();
        VcxError::from_msg(kind_num.into(), utils::error::error_message(&aries_err.kind().clone().into()))
    }

}

impl<T> From<sync::PoisonError<T>> for VcxError {
    fn from(_: sync::PoisonError<T>) -> Self {
        VcxError { inner: Context::new(Backtrace::new()).context(VcxErrorKind::PoisonedLock) }
    }
}

impl From<Context<VcxErrorKind>> for VcxError {
    fn from(inner: Context<VcxErrorKind>) -> VcxError {
        VcxError { inner }
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
