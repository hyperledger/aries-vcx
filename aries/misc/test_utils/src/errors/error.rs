use thiserror::Error as ThisError;

pub type TestUtilsResult<T> = Result<T, TestUtilsError>;

#[derive(Debug, ThisError)]
pub enum TestUtilsError {
    #[error("Logging error: {0}")]
    LoggingError(String),
    #[error("Unknown error: {0}")]
    UnknownError(String),
}
