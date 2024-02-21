use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
};

use crate::cl::{Error as CryptoError, ErrorKind as CryptoErrorKind};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Input,
    InvalidState,
    Unexpected,
    ProofRejected,
    ConversionError,
    ValidationError,
}

impl ErrorKind {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Input => "Input error",
            Self::InvalidState => "Invalid state",
            Self::Unexpected => "Unexpected error",
            Self::ProofRejected => "Proof rejected",
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

    #[must_use]
    #[inline]
    pub const fn kind(&self) -> ErrorKind {
        self.kind
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
