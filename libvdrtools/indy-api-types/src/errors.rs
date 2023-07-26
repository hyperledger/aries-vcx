use std::{
    cell,
    cell::RefCell,
    ffi::{CString, NulError},
    fmt, io, ptr,
    sync::Arc,
};

use log;
use std::error::Error;
use thiserror::Error as ThisError;

#[cfg(feature = "casting_errors_wallet")]
use sqlx;

#[cfg(feature = "casting_errors_misc")]
use ursa::errors::{UrsaCryptoError, UrsaCryptoErrorKind};

use libc::c_char;

use crate::ErrorCode;

pub mod prelude {
    pub use super::{
        err_msg, get_current_error_c_json, set_current_error, IndyError, IndyErrorExt,
        IndyErrorKind, IndyResult, IndyResultExt,
    };
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, ThisError)]
pub enum IndyErrorKind {
    // Common errors
    #[error("Invalid library state")]
    InvalidState,
    #[error("Invalid structure")]
    InvalidStructure,
    #[error("Invalid parameter {0}")]
    InvalidParam(u32),
    #[error("IO error")]
    IOError,
    // Anoncreds errors
    #[error("Duplicated master secret")]
    MasterSecretDuplicateName,
    #[error("Proof rejected")]
    ProofRejected,
    #[error("Revocation registry is full")]
    RevocationRegistryFull,
    #[error("Invalid revocation id")]
    InvalidUserRevocId,
    #[error("Credential revoked")]
    CredentialRevoked,
    #[error("Credential definition already exists")]
    CredDefAlreadyExists,
    // Ledger errors
    #[error("No consensus")]
    NoConsensus,
    #[error("Invalid transaction")]
    InvalidTransaction,
    #[error("Item not found on ledger")]
    LedgerItemNotFound,
    // Pool errors
    #[error("Pool not created")]
    PoolNotCreated,
    #[error("Invalid pool handle")]
    InvalidPoolHandle,
    #[error("Pool work terminated")]
    PoolTerminated,
    #[error("Pool timeout")]
    PoolTimeout,
    #[error("Pool ledger config already exists")]
    PoolConfigAlreadyExists,
    #[error("Pool Genesis Transactions are not compatible with Protocol version")]
    PoolIncompatibleProtocolVersion,
    // Crypto errors
    #[error("Unknown crypto")]
    UnknownCrypto,
    // Wallet errors
    #[error("Invalid wallet handle was passed")]
    InvalidWalletHandle,
    #[error("Unknown wallet storage type")]
    UnknownWalletStorageType,
    #[error("Wallet storage type already registered")]
    WalletStorageTypeAlreadyRegistered,
    #[error("Wallet with this name already exists")]
    WalletAlreadyExists,
    #[error("Wallet not found")]
    WalletNotFound,
    #[error("Wallet already opened")]
    WalletAlreadyOpened,
    #[error("Wallet security error")]
    WalletAccessFailed,
    #[error("Wallet encoding error")]
    WalletEncodingError,
    #[error("Wallet storage error occurred")]
    WalletStorageError,
    #[error("Wallet encryption error")]
    WalletEncryptionError,
    #[error("Wallet item not found")]
    WalletItemNotFound,
    #[error("Wallet item already exists")]
    WalletItemAlreadyExists,
    #[error("Wallet query error")]
    WalletQueryError,
    // DID errors
    #[error("DID already exists")]
    DIDAlreadyExists,
    // Payments errors
    #[error("Unknown payment method type")]
    UnknownPaymentMethodType,
    #[error("No method were scraped from inputs/outputs or more than one were scraped")]
    IncompatiblePaymentMethods,
    #[error("Payment insufficient funds on inputs")]
    PaymentInsufficientFunds,
    #[error("Payment Source does not exist")]
    PaymentSourceDoesNotExist,
    #[error("Payment operation not supported")]
    PaymentOperationNotSupported,
    #[error("Payment extra funds")]
    PaymentExtraFunds,
    #[error("The transaction is not allowed to a requester")]
    TransactionNotAllowed,
    #[error("Query account does not exist")]
    QueryAccountDoesNotExist,

    #[error("Invalid VDR handle")]
    InvalidVDRHandle,
    #[error("Failed to get ledger for VDR Namespace")]
    InvalidVDRNamespace,
    #[error("Registered Ledger type does not match to the network of id")]
    IncompatibleLedger,
}

#[derive(Debug, Clone, ThisError)]
pub struct IndyError {
    // FIXME: We have to use Arc as for now we clone messages in pool service
    // FIXME: In theory we can avoid sync by refactoring of pool service
    #[source]
    kind: IndyErrorKind,
    msg: Arc<String>,
}

impl fmt::Display for IndyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Error: {}", self.kind())?;

        if let Some(src) = self.kind.source() {
            writeln!(f, "  Caused by: {}", src)?;
        }

        Ok(())
    }
}

impl IndyError {
    pub fn from_msg<D>(kind: IndyErrorKind, msg: D) -> IndyError
    where
        D: fmt::Display + Send + Sync + 'static,
    {
        IndyError {
            kind,
            msg: Arc::new(msg.to_string()),
        }
    }

    pub fn kind(&self) -> IndyErrorKind {
        self.kind
    }

    pub fn extend<D>(self, msg: D) -> IndyError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        IndyError {
            kind: self.kind,
            msg: Arc::new(format!("{}\n  Caused by: {msg}", self.msg)),
        }
    }

    pub fn map<D>(self, kind: IndyErrorKind, msg: D) -> IndyError
    where
        D: fmt::Display + Send + Sync + 'static,
    {
        IndyError {
            kind,
            msg: Arc::new(format!("{}\n  Caused by: {msg}", self.msg)),
        }
    }
}

pub fn err_msg<D>(kind: IndyErrorKind, msg: D) -> IndyError
where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    IndyError::from_msg(kind, msg)
}

impl From<IndyErrorKind> for IndyError {
    fn from(kind: IndyErrorKind) -> IndyError {
        IndyError {
            kind,
            msg: Arc::new(String::new()),
        }
    }
}

impl From<io::Error> for IndyError {
    fn from(err: io::Error) -> Self {
        IndyError {
            kind: IndyErrorKind::IOError,
            msg: Arc::new(err.to_string()),
        }
    }
}

#[cfg(feature = "casting_errors_ledger")]
impl From<zmq::Error> for IndyError {
    fn from(err: zmq::Error) -> Self {
        IndyError {
            kind: IndyErrorKind::IOError,
            msg: Arc::new(err.to_string()),
        }
    }
}

impl From<cell::BorrowError> for IndyError {
    fn from(err: cell::BorrowError) -> Self {
        IndyError {
            kind: IndyErrorKind::InvalidState,
            msg: Arc::new(err.to_string()),
        }
    }
}

impl From<cell::BorrowMutError> for IndyError {
    fn from(err: cell::BorrowMutError) -> Self {
        IndyError {
            kind: IndyErrorKind::InvalidState,
            msg: Arc::new(err.to_string()),
        }
    }
}

impl From<futures::channel::oneshot::Canceled> for IndyError {
    fn from(err: futures::channel::oneshot::Canceled) -> Self {
        IndyError {
            kind: IndyErrorKind::InvalidState,
            msg: Arc::new(err.to_string()),
        }
    }
}

impl From<log::SetLoggerError> for IndyError {
    fn from(err: log::SetLoggerError) -> IndyError {
        IndyError {
            kind: IndyErrorKind::InvalidState,
            msg: Arc::new(err.to_string()),
        }
    }
}

#[cfg(feature = "casting_errors_misc")]
impl From<UrsaCryptoError> for IndyError {
    fn from(err: UrsaCryptoError) -> Self {
        match err.kind() {
            UrsaCryptoErrorKind::InvalidState => {
                IndyError::from_msg(IndyErrorKind::InvalidState, err)
            }
            UrsaCryptoErrorKind::InvalidStructure => {
                IndyError::from_msg(IndyErrorKind::InvalidStructure, err)
            }
            UrsaCryptoErrorKind::IOError => IndyError::from_msg(IndyErrorKind::IOError, err),
            UrsaCryptoErrorKind::InvalidRevocationAccumulatorIndex => {
                IndyError::from_msg(IndyErrorKind::InvalidUserRevocId, err)
            }
            UrsaCryptoErrorKind::RevocationAccumulatorIsFull => {
                IndyError::from_msg(IndyErrorKind::RevocationRegistryFull, err)
            }
            UrsaCryptoErrorKind::ProofRejected => {
                IndyError::from_msg(IndyErrorKind::ProofRejected, err)
            }
            UrsaCryptoErrorKind::CredentialRevoked => {
                IndyError::from_msg(IndyErrorKind::CredentialRevoked, err)
            }
            UrsaCryptoErrorKind::InvalidParam(_) => {
                IndyError::from_msg(IndyErrorKind::InvalidStructure, err)
            }
        }
    }
}

#[cfg(feature = "casting_errors_misc")]
impl From<bs58::decode::Error> for IndyError {
    fn from(_err: bs58::decode::Error) -> Self {
        IndyError::from_msg(
            IndyErrorKind::InvalidStructure,
            "The base58 input contained a character not part of the base58 alphabet",
        )
    }
}

#[cfg(feature = "casting_errors_misc")]
impl From<openssl::error::ErrorStack> for IndyError {
    fn from(err: openssl::error::ErrorStack) -> IndyError {
        // TODO: FIXME: Analyze ErrorStack and split invalid structure errors from other errors
        err.to_indy(IndyErrorKind::InvalidState, "Internal OpenSSL error")
    }
}

#[cfg(feature = "casting_errors_wallet")]
impl From<sqlx::Error> for IndyError {
    fn from(err: sqlx::Error) -> IndyError {
        match &err {
            sqlx::Error::RowNotFound => {
                err.to_indy(IndyErrorKind::WalletItemNotFound, "Item not found")
            }
            sqlx::Error::Database(e) => match e.code() {
                Some(code) => match code.as_ref() {
                    // Constraint unuque - sqlite (2067)
                    "2067" => err.to_indy(
                        IndyErrorKind::WalletItemAlreadyExists,
                        "Wallet item already exists",
                    ),
                    // Integrity constraint violation (23000)
                    "23000" => err.to_indy(
                        IndyErrorKind::WalletItemAlreadyExists,
                        "Wallet item already exists",
                    ),
                    _ => err.to_indy(IndyErrorKind::InvalidState, "Unexpected database error"),
                },
                None => err.to_indy(IndyErrorKind::InvalidState, "Unexpected database error"),
            },
            sqlx::Error::Io(_) => err.to_indy(
                IndyErrorKind::IOError,
                "IO error during access sqlite database",
            ),
            sqlx::Error::Tls(_) => err.to_indy(
                IndyErrorKind::IOError,
                "IO error during access sqlite database",
            ),
            _ => err.to_indy(IndyErrorKind::InvalidState, "Unexpected database error"),
        }
    }
}

impl From<NulError> for IndyError {
    fn from(err: NulError) -> IndyError {
        err.to_indy(
            IndyErrorKind::InvalidState,
            "Null symbols in payments strings",
        ) // TODO: Review kind
    }
}

impl<T> From<IndyResult<T>> for ErrorCode {
    fn from(r: Result<T, IndyError>) -> ErrorCode {
        match r {
            Ok(_) => ErrorCode::Success,
            Err(err) => err.into(),
        }
    }
}

impl From<IndyError> for ErrorCode {
    fn from(err: IndyError) -> ErrorCode {
        set_current_error(&err);
        err.kind().into()
    }
}

impl From<IndyErrorKind> for ErrorCode {
    fn from(code: IndyErrorKind) -> ErrorCode {
        match code {
            IndyErrorKind::InvalidState => ErrorCode::CommonInvalidState,
            IndyErrorKind::InvalidStructure => ErrorCode::CommonInvalidStructure,
            IndyErrorKind::InvalidParam(num) => match num {
                1 => ErrorCode::CommonInvalidParam1,
                2 => ErrorCode::CommonInvalidParam2,
                3 => ErrorCode::CommonInvalidParam3,
                4 => ErrorCode::CommonInvalidParam4,
                5 => ErrorCode::CommonInvalidParam5,
                6 => ErrorCode::CommonInvalidParam6,
                7 => ErrorCode::CommonInvalidParam7,
                8 => ErrorCode::CommonInvalidParam8,
                9 => ErrorCode::CommonInvalidParam9,
                10 => ErrorCode::CommonInvalidParam10,
                11 => ErrorCode::CommonInvalidParam11,
                12 => ErrorCode::CommonInvalidParam12,
                13 => ErrorCode::CommonInvalidParam13,
                14 => ErrorCode::CommonInvalidParam14,
                15 => ErrorCode::CommonInvalidParam15,
                16 => ErrorCode::CommonInvalidParam16,
                17 => ErrorCode::CommonInvalidParam17,
                18 => ErrorCode::CommonInvalidParam18,
                19 => ErrorCode::CommonInvalidParam19,
                20 => ErrorCode::CommonInvalidParam20,
                21 => ErrorCode::CommonInvalidParam21,
                22 => ErrorCode::CommonInvalidParam22,
                23 => ErrorCode::CommonInvalidParam23,
                24 => ErrorCode::CommonInvalidParam24,
                25 => ErrorCode::CommonInvalidParam25,
                26 => ErrorCode::CommonInvalidParam26,
                27 => ErrorCode::CommonInvalidParam27,
                _ => ErrorCode::CommonInvalidState,
            },
            IndyErrorKind::IOError => ErrorCode::CommonIOError,
            IndyErrorKind::MasterSecretDuplicateName => {
                ErrorCode::AnoncredsMasterSecretDuplicateNameError
            }
            IndyErrorKind::ProofRejected => ErrorCode::AnoncredsProofRejected,
            IndyErrorKind::RevocationRegistryFull => {
                ErrorCode::AnoncredsRevocationRegistryFullError
            }
            IndyErrorKind::InvalidUserRevocId => ErrorCode::AnoncredsInvalidUserRevocId,
            IndyErrorKind::CredentialRevoked => ErrorCode::AnoncredsCredentialRevoked,
            IndyErrorKind::CredDefAlreadyExists => ErrorCode::AnoncredsCredDefAlreadyExistsError,
            IndyErrorKind::NoConsensus => ErrorCode::LedgerNoConsensusError,
            IndyErrorKind::InvalidTransaction => ErrorCode::LedgerInvalidTransaction,
            IndyErrorKind::LedgerItemNotFound => ErrorCode::LedgerNotFound,
            IndyErrorKind::PoolNotCreated => ErrorCode::PoolLedgerNotCreatedError,
            IndyErrorKind::InvalidPoolHandle => ErrorCode::PoolLedgerInvalidPoolHandle,
            IndyErrorKind::PoolTerminated => ErrorCode::PoolLedgerTerminated,
            IndyErrorKind::PoolTimeout => ErrorCode::PoolLedgerTimeout,
            IndyErrorKind::PoolConfigAlreadyExists => ErrorCode::PoolLedgerConfigAlreadyExistsError,
            IndyErrorKind::PoolIncompatibleProtocolVersion => {
                ErrorCode::PoolIncompatibleProtocolVersion
            }
            IndyErrorKind::UnknownCrypto => ErrorCode::UnknownCryptoTypeError,
            IndyErrorKind::InvalidWalletHandle => ErrorCode::WalletInvalidHandle,
            IndyErrorKind::UnknownWalletStorageType => ErrorCode::WalletUnknownTypeError,
            IndyErrorKind::WalletStorageTypeAlreadyRegistered => {
                ErrorCode::WalletTypeAlreadyRegisteredError
            }
            IndyErrorKind::WalletAlreadyExists => ErrorCode::WalletAlreadyExistsError,
            IndyErrorKind::WalletNotFound => ErrorCode::WalletNotFoundError,
            IndyErrorKind::WalletAlreadyOpened => ErrorCode::WalletAlreadyOpenedError,
            IndyErrorKind::WalletAccessFailed => ErrorCode::WalletAccessFailed,
            IndyErrorKind::WalletEncodingError => ErrorCode::WalletDecodingError,
            IndyErrorKind::WalletStorageError => ErrorCode::WalletStorageError,
            IndyErrorKind::WalletEncryptionError => ErrorCode::WalletEncryptionError,
            IndyErrorKind::WalletItemNotFound => ErrorCode::WalletItemNotFound,
            IndyErrorKind::WalletItemAlreadyExists => ErrorCode::WalletItemAlreadyExists,
            IndyErrorKind::WalletQueryError => ErrorCode::WalletQueryError,
            IndyErrorKind::DIDAlreadyExists => ErrorCode::DidAlreadyExistsError,
            IndyErrorKind::UnknownPaymentMethodType => ErrorCode::PaymentUnknownMethodError,
            IndyErrorKind::IncompatiblePaymentMethods => ErrorCode::PaymentIncompatibleMethodsError,
            IndyErrorKind::PaymentInsufficientFunds => ErrorCode::PaymentInsufficientFundsError,
            IndyErrorKind::PaymentSourceDoesNotExist => ErrorCode::PaymentSourceDoesNotExistError,
            IndyErrorKind::PaymentOperationNotSupported => {
                ErrorCode::PaymentOperationNotSupportedError
            }
            IndyErrorKind::PaymentExtraFunds => ErrorCode::PaymentExtraFundsError,
            IndyErrorKind::TransactionNotAllowed => ErrorCode::TransactionNotAllowedError,
            IndyErrorKind::QueryAccountDoesNotExist => ErrorCode::QueryAccountDoesNotexistError,
            IndyErrorKind::InvalidVDRHandle => ErrorCode::InvalidVDRHandle,
            IndyErrorKind::InvalidVDRNamespace => ErrorCode::InvalidVDRNamespace,
            IndyErrorKind::IncompatibleLedger => ErrorCode::IncompatibleLedger,
        }
    }
}

impl From<ErrorCode> for IndyResult<()> {
    fn from(err: ErrorCode) -> IndyResult<()> {
        if err == ErrorCode::Success {
            Ok(())
        } else {
            Err(err.into())
        }
    }
}

impl From<ErrorCode> for IndyError {
    fn from(err: ErrorCode) -> IndyError {
        err_msg(err.into(), "Plugin returned error".to_string())
    }
}

impl From<ErrorCode> for IndyErrorKind {
    fn from(err: ErrorCode) -> IndyErrorKind {
        match err {
            ErrorCode::CommonInvalidState => IndyErrorKind::InvalidState,
            ErrorCode::CommonInvalidStructure => IndyErrorKind::InvalidStructure,
            ErrorCode::CommonInvalidParam1 => IndyErrorKind::InvalidParam(1),
            ErrorCode::CommonInvalidParam2 => IndyErrorKind::InvalidParam(2),
            ErrorCode::CommonInvalidParam3 => IndyErrorKind::InvalidParam(3),
            ErrorCode::CommonInvalidParam4 => IndyErrorKind::InvalidParam(4),
            ErrorCode::CommonInvalidParam5 => IndyErrorKind::InvalidParam(5),
            ErrorCode::CommonInvalidParam6 => IndyErrorKind::InvalidParam(6),
            ErrorCode::CommonInvalidParam7 => IndyErrorKind::InvalidParam(7),
            ErrorCode::CommonInvalidParam8 => IndyErrorKind::InvalidParam(8),
            ErrorCode::CommonInvalidParam9 => IndyErrorKind::InvalidParam(9),
            ErrorCode::CommonInvalidParam10 => IndyErrorKind::InvalidParam(10),
            ErrorCode::CommonInvalidParam11 => IndyErrorKind::InvalidParam(11),
            ErrorCode::CommonInvalidParam12 => IndyErrorKind::InvalidParam(12),
            ErrorCode::CommonInvalidParam13 => IndyErrorKind::InvalidParam(13),
            ErrorCode::CommonInvalidParam14 => IndyErrorKind::InvalidParam(14),
            ErrorCode::CommonInvalidParam15 => IndyErrorKind::InvalidParam(15),
            ErrorCode::CommonInvalidParam16 => IndyErrorKind::InvalidParam(16),
            ErrorCode::CommonInvalidParam17 => IndyErrorKind::InvalidParam(17),
            ErrorCode::CommonInvalidParam18 => IndyErrorKind::InvalidParam(18),
            ErrorCode::CommonInvalidParam19 => IndyErrorKind::InvalidParam(19),
            ErrorCode::CommonInvalidParam20 => IndyErrorKind::InvalidParam(20),
            ErrorCode::CommonInvalidParam21 => IndyErrorKind::InvalidParam(21),
            ErrorCode::CommonInvalidParam22 => IndyErrorKind::InvalidParam(22),
            ErrorCode::CommonInvalidParam23 => IndyErrorKind::InvalidParam(23),
            ErrorCode::CommonInvalidParam24 => IndyErrorKind::InvalidParam(24),
            ErrorCode::CommonInvalidParam25 => IndyErrorKind::InvalidParam(25),
            ErrorCode::CommonInvalidParam26 => IndyErrorKind::InvalidParam(26),
            ErrorCode::CommonInvalidParam27 => IndyErrorKind::InvalidParam(27),
            ErrorCode::CommonIOError => IndyErrorKind::IOError,
            ErrorCode::AnoncredsMasterSecretDuplicateNameError => {
                IndyErrorKind::MasterSecretDuplicateName
            }
            ErrorCode::AnoncredsProofRejected => IndyErrorKind::ProofRejected,
            ErrorCode::AnoncredsRevocationRegistryFullError => {
                IndyErrorKind::RevocationRegistryFull
            }
            ErrorCode::AnoncredsInvalidUserRevocId => IndyErrorKind::InvalidUserRevocId,
            ErrorCode::AnoncredsCredentialRevoked => IndyErrorKind::CredentialRevoked,
            ErrorCode::AnoncredsCredDefAlreadyExistsError => IndyErrorKind::CredDefAlreadyExists,
            ErrorCode::LedgerNoConsensusError => IndyErrorKind::NoConsensus,
            ErrorCode::LedgerInvalidTransaction => IndyErrorKind::InvalidTransaction,
            ErrorCode::LedgerNotFound => IndyErrorKind::LedgerItemNotFound,
            ErrorCode::PoolLedgerNotCreatedError => IndyErrorKind::PoolNotCreated,
            ErrorCode::PoolLedgerInvalidPoolHandle => IndyErrorKind::InvalidPoolHandle,
            ErrorCode::PoolLedgerTerminated => IndyErrorKind::PoolTerminated,
            ErrorCode::PoolLedgerTimeout => IndyErrorKind::PoolTimeout,
            ErrorCode::PoolLedgerConfigAlreadyExistsError => IndyErrorKind::PoolConfigAlreadyExists,
            ErrorCode::PoolIncompatibleProtocolVersion => {
                IndyErrorKind::PoolIncompatibleProtocolVersion
            }
            ErrorCode::UnknownCryptoTypeError => IndyErrorKind::UnknownCrypto,
            ErrorCode::WalletInvalidHandle => IndyErrorKind::InvalidWalletHandle,
            ErrorCode::WalletUnknownTypeError => IndyErrorKind::UnknownWalletStorageType,
            ErrorCode::WalletTypeAlreadyRegisteredError => {
                IndyErrorKind::WalletStorageTypeAlreadyRegistered
            }
            ErrorCode::WalletAlreadyExistsError => IndyErrorKind::WalletAlreadyExists,
            ErrorCode::WalletNotFoundError => IndyErrorKind::WalletNotFound,
            ErrorCode::WalletAlreadyOpenedError => IndyErrorKind::WalletAlreadyOpened,
            ErrorCode::WalletAccessFailed => IndyErrorKind::WalletAccessFailed,
            ErrorCode::WalletDecodingError => IndyErrorKind::WalletEncodingError,
            ErrorCode::WalletStorageError => IndyErrorKind::WalletStorageError,
            ErrorCode::WalletEncryptionError => IndyErrorKind::WalletEncryptionError,
            ErrorCode::WalletItemNotFound => IndyErrorKind::WalletItemNotFound,
            ErrorCode::WalletItemAlreadyExists => IndyErrorKind::WalletItemAlreadyExists,
            ErrorCode::WalletQueryError => IndyErrorKind::WalletQueryError,
            ErrorCode::DidAlreadyExistsError => IndyErrorKind::DIDAlreadyExists,
            ErrorCode::PaymentUnknownMethodError => IndyErrorKind::UnknownPaymentMethodType,
            ErrorCode::PaymentIncompatibleMethodsError => IndyErrorKind::IncompatiblePaymentMethods,
            ErrorCode::PaymentInsufficientFundsError => IndyErrorKind::PaymentInsufficientFunds,
            ErrorCode::PaymentSourceDoesNotExistError => IndyErrorKind::PaymentSourceDoesNotExist,
            ErrorCode::PaymentOperationNotSupportedError => {
                IndyErrorKind::PaymentOperationNotSupported
            }
            ErrorCode::PaymentExtraFundsError => IndyErrorKind::PaymentExtraFunds,
            ErrorCode::TransactionNotAllowedError => IndyErrorKind::TransactionNotAllowed,
            ErrorCode::InvalidVDRHandle => IndyErrorKind::InvalidVDRHandle,
            ErrorCode::InvalidVDRNamespace => IndyErrorKind::InvalidVDRNamespace,
            ErrorCode::IncompatibleLedger => IndyErrorKind::IncompatibleLedger,
            _code => IndyErrorKind::InvalidState,
        }
    }
}

pub type IndyResult<T> = Result<T, IndyError>;

/// Extension methods for `Result`.
pub trait IndyResultExt<T, E> {
    fn to_indy<D>(self, kind: IndyErrorKind, msg: D) -> IndyResult<T>
    where
        D: fmt::Display + Send + Sync + 'static;
}

impl<T, E> IndyResultExt<T, E> for Result<T, E>
where
    E: fmt::Display,
{
    fn to_indy<D>(self, kind: IndyErrorKind, msg: D) -> IndyResult<T>
    where
        D: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| err.to_indy(kind, msg))
    }
}

/// Extension methods for `Error`.
pub trait IndyErrorExt {
    fn to_indy<D>(self, kind: IndyErrorKind, msg: D) -> IndyError
    where
        D: fmt::Display + Send + Sync + 'static;
}

impl<E> IndyErrorExt for E
where
    E: fmt::Display,
{
    fn to_indy<D>(self, kind: IndyErrorKind, msg: D) -> IndyError
    where
        D: fmt::Display + Send + Sync + 'static,
    {
        IndyError::from_msg(kind, format!("{msg}\n  Caused by: {self}"))
    }
}

thread_local! {
    pub static CURRENT_ERROR_C_JSON: RefCell<Option<CString>> = RefCell::new(None);
}

pub fn set_current_error(err: &IndyError) {
    CURRENT_ERROR_C_JSON
        .try_with(|error| {
            let error_json = json!({
                "message": err.to_string(),
                "backtrace": err.source().map(|bt| bt.to_string())
            })
            .to_string();
            error.replace(Some(string_to_cstring(error_json)));
        })
        .map_err(|err| error!("Thread local variable access failed with: {:?}", err))
        .ok();
}

/// Get details for last occurred error.
///
/// This function should be called in two places to handle both cases of error occurrence:
///     1) synchronous  - in the same application thread
///     2) asynchronous - inside of function callback
///
/// NOTE: Error is stored until the next one occurs in the same execution thread or until asynchronous callback finished.
///       Returning pointer has the same lifetime.
///
/// #Params
/// * `error_json_p` - Reference that will contain error details (if any error has occurred before)
///  in the format:
/// {
///     "backtrace": Optional<str> - error backtrace.
///         Collecting of backtrace can be enabled by:
///             1) setting environment variable `RUST_BACKTRACE=1`
///             2) calling `indy_set_runtime_config` API function with `collect_backtrace: true`
///     "message": str - human-readable error description
/// }
///
pub fn get_current_error_c_json() -> *const c_char {
    let mut value = ptr::null();

    CURRENT_ERROR_C_JSON
        .try_with(|err| err.borrow().as_ref().map(|err| value = err.as_ptr()))
        .map_err(|err| error!("Thread local variable access failed with: {:?}", err))
        .ok();

    value
}

pub fn string_to_cstring(s: String) -> CString {
    CString::new(s).unwrap()
}
