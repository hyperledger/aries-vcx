use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibvcxLoggerError {
    #[error("Cannot init logger {0}")]
    LoggingError(#[from] log::SetLoggerError),
}

pub type LibvcxLoggerResult<T> = Result<T, LibvcxLoggerError>;
