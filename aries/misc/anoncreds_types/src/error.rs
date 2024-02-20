use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    result::Result as StdResult,
};

use crate::cl::{Error as CryptoError, ErrorKind as CryptoErrorKind};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    // General errors
    Input,
    IOError,
    InvalidState,
    Unexpected,
    // Credential/proof errors
    CredentialRevoked,
    InvalidUserRevocId,
    ProofRejected,
    RevocationRegistryFull,
    ConversionError,
    ValidationError,
}

impl ErrorKind {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Input => "Input error",
            Self::IOError => "IO error",
            Self::InvalidState => "Invalid state",
            Self::Unexpected => "Unexpected error",
            Self::CredentialRevoked => "Credential revoked",
            Self::InvalidUserRevocId => "Invalid revocation accumulator index",
            Self::ProofRejected => "Proof rejected",
            Self::RevocationRegistryFull => "Revocation registry full",
            Self::ConversionError => "Conversion error",
            Self::ValidationError => "Validation error",
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The standard crate error type
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    pub cause: Option<Box<dyn StdError + Send + Sync + 'static>>,
    pub message: Option<String>,
    // backtrace (when supported)
}

impl Error {
    pub fn from_msg<T: Into<String>>(kind: ErrorKind, msg: T) -> Self {
        Self {
            kind,
            cause: None,
            message: Some(msg.into()),
        }
    }

    pub fn from_opt_msg<T: Into<String>>(kind: ErrorKind, msg: Option<T>) -> Self {
        Self {
            kind,
            cause: None,
            message: msg.map(Into::into),
        }
    }

    #[must_use]
    #[inline]
    pub const fn kind(&self) -> ErrorKind {
        self.kind
    }

    #[must_use]
    pub fn with_cause<T: Into<Box<dyn StdError + Send + Sync>>>(mut self, err: T) -> Self {
        self.cause = Some(err.into());
        self
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.kind, &self.message) {
            (ErrorKind::Input, None) => write!(f, "{}", self.kind),
            (ErrorKind::Input, Some(msg)) => f.write_str(msg),
            (kind, None) => write!(f, "{kind}"),
            (kind, Some(msg)) => write!(f, "{kind}: {msg}"),
        }?;
        if let Some(ref source) = self.cause {
            write!(f, " [{source}]")?;
        }
        Ok(())
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.cause
            .as_ref()
            .map(|err| unsafe { std::mem::transmute(&**err) })
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.message == other.message
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self {
            kind,
            cause: None,
            message: None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::from(ErrorKind::IOError).with_cause(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        // FIXME could be input or output...
        Self::from(ErrorKind::Input).with_cause(err)
    }
}

impl From<CryptoError> for Error {
    fn from(err: CryptoError) -> Self {
        let message = err.to_string();
        let kind = match err.kind() {
            CryptoErrorKind::InvalidState => ErrorKind::InvalidState,
            CryptoErrorKind::ProofRejected => ErrorKind::ProofRejected,
        };
        Self::from_msg(kind, message)
    }
}

impl<M> From<(ErrorKind, M)> for Error
where
    M: fmt::Display + Send + Sync + 'static,
{
    fn from((kind, msg): (ErrorKind, M)) -> Self {
        Self::from_msg(kind, msg.to_string())
    }
}

macro_rules! err_msg {
    () => {
        $crate::error::Error::from($crate::error::ErrorKind::Input)
    };
    ($kind:ident) => {
        $crate::error::Error::from($crate::error::ErrorKind::$kind)
    };
    ($kind:ident, $($args:tt)+) => {
        $crate::error::Error::from_msg($crate::error::ErrorKind::$kind, format!($($args)+))
    };
    ($($args:tt)+) => {
        $crate::error::Error::from_msg($crate::error::ErrorKind::Input, format!($($args)+))
    };
}

pub trait ResultExt<T, E> {
    fn map_err_string(self) -> StdResult<T, String>;
    fn map_input_err<F, M>(self, mapfn: F) -> Result<T>
    where
        F: FnOnce() -> M,
        M: fmt::Display + Send + Sync + 'static;
    fn with_err_msg<M>(self, kind: ErrorKind, msg: M) -> Result<T>
    where
        M: fmt::Display + Send + Sync + 'static;
    fn with_input_err<M>(self, msg: M) -> Result<T>
    where
        M: fmt::Display + Send + Sync + 'static;
}

impl<T, E> ResultExt<T, E> for StdResult<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn map_err_string(self) -> StdResult<T, String> {
        self.map_err(|err| err.to_string())
    }

    fn map_input_err<F, M>(self, mapfn: F) -> Result<T>
    where
        F: FnOnce() -> M,
        M: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| Error::from_msg(ErrorKind::Input, mapfn().to_string()).with_cause(err))
    }

    fn with_err_msg<M>(self, kind: ErrorKind, msg: M) -> Result<T>
    where
        M: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| Error::from_msg(kind, msg.to_string()).with_cause(err))
    }

    #[inline]
    fn with_input_err<M>(self, msg: M) -> Result<T>
    where
        M: fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|err| Error::from_msg(ErrorKind::Input, msg.to_string()).with_cause(err))
    }
}
