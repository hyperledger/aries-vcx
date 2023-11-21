use serde_json::Error as JsonError;
use thiserror::Error as ThisError;
use vdrtools::IndyError;

pub type MigrationResult<T> = Result<T, MigrationError>;

#[derive(Debug, ThisError)]
pub enum MigrationError {
    #[error("JSON error: {0}")]
    Json(#[from] JsonError),
    #[error("Indy error: {0}")]
    Indy(#[from] IndyError),
    #[error("Source and destination wallets must be different!")]
    EqualWalletHandles,
}
