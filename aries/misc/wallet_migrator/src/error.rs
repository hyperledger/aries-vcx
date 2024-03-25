use aries_vcx_wallet::errors::error::VcxWalletError;
use serde_json::Error as JsonError;
use thiserror::Error as ThisError;

pub type MigrationResult<T> = Result<T, MigrationError>;

#[derive(Debug, ThisError)]
pub enum MigrationError {
    #[error("JSON error: {0}")]
    Json(#[from] JsonError),
    #[error("VcxWallet error: {0}")]
    VcxWallet(#[from] VcxWalletError),
    #[error("Unsupported wallet migration")]
    Unsupported,
}
