use std::{error::Error, fmt};

#[derive(Copy, Clone, Eq, PartialEq, Debug, thiserror::Error)]
pub enum ValidationErrorKind {
    #[error("Invalid DID")]
    InvalidDid,
    #[error("Invalid VERKEY")]
    InvalidVerkey,
    #[error("Value needs to be base58")]
    NotBase58,
}

#[derive(Debug, thiserror::Error)]
pub struct ValidationError {
    msg: String,
    kind: ValidationErrorKind,
}

impl fmt::Display for ValidationError {
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

impl ValidationError {
    pub fn from_msg<D>(kind: ValidationErrorKind, msg: D) -> ValidationError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        ValidationError {
            msg: msg.to_string(),
            kind,
        }
    }

    pub fn kind(&self) -> ValidationErrorKind {
        self.kind
    }
}

pub fn err_msg<D>(kind: ValidationErrorKind, msg: D) -> ValidationError
where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    ValidationError::from_msg(kind, msg)
}

pub type ValidationResult<T> = Result<T, ValidationError>;
