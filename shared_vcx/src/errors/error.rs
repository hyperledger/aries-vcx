use std::{error::Error, fmt};

#[derive(Copy, Clone, Eq, PartialEq, Debug, thiserror::Error)]
pub enum SharedVcxErrorKind {
    #[error("Invalid DID")]
    InvalidDid,
    #[error("Invalid VERKEY")]
    InvalidVerkey,
    #[error("Value needs to be base58")]
    NotBase58,
}

#[derive(Debug, thiserror::Error)]
pub struct SharedVcxError {
    msg: String,
    kind: SharedVcxErrorKind,
}

impl fmt::Display for SharedVcxError {
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

impl SharedVcxError {
    pub fn from_msg<D>(kind: SharedVcxErrorKind, msg: D) -> SharedVcxError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        SharedVcxError {
            msg: msg.to_string(),
            kind,
        }
    }

    pub fn kind(&self) -> SharedVcxErrorKind {
        self.kind
    }
}

pub fn err_msg<D>(kind: SharedVcxErrorKind, msg: D) -> SharedVcxError
where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    SharedVcxError::from_msg(kind, msg)
}

pub type SharedVcxResult<T> = Result<T, SharedVcxError>;
