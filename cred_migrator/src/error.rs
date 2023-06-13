use serde_json::Error as JsonError;
use thiserror::Error as ThisError;
use vdrtools::IndyError;

pub type MigrationResult<T> = Result<T, MigrationError>;

#[derive(Debug, ThisError)]
pub enum MigrationError {
    #[error("could not serialize/deserialize record value")]
    Json(#[from] JsonError),
    #[error(transparent)]
    Indy(#[from] IndyError),
}
