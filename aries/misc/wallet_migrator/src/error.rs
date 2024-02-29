use aries_vcx_core::errors::error::AriesVcxCoreError;
use serde_json::Error as JsonError;
use thiserror::Error as ThisError;

pub type MigrationResult<T> = Result<T, MigrationError>;

#[derive(Debug, ThisError)]
pub enum MigrationError {
    #[error("JSON error: {0}")]
    Json(#[from] JsonError),
    #[error("VcxCore error: {0}")]
    VcxCore(#[from] AriesVcxCoreError),
    #[error("Unsupported wallet migration")]
    Unsupported,
}
