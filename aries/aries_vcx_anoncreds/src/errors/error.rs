use aries_vcx_wallet::errors::error::VcxWalletError;
use thiserror::Error as ThisError;

pub type VcxAnoncredsResult<T> = Result<T, VcxAnoncredsError>;

#[derive(Debug, ThisError)]
pub enum VcxAnoncredsError {
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Invalid state: {0}")]
    InvalidState(String),
    #[error("Wallet error: {0}")]
    WalletError(#[from] VcxWalletError),
    #[error("Ursa error: {0}")]
    UrsaError(String),
    #[error("IO error: {0}")]
    IOError(String),
    #[error("Unknown error: {0}")]
    UnknownError(String),
    #[error("Proof rejected: {0}")]
    ProofRejected(String),
    #[error("Action not supported: {0}")]
    ActionNotSupported(String),
    #[error("Invalid proof request: {0}")]
    InvalidProofRequest(String),
    #[error("Invalid attributes structure: {0}")]
    InvalidAttributesStructure(String),
    #[error("Invalid option: {0}")]
    InvalidOption(String),
    #[error("Invalid schema: {0}")]
    InvalidSchema(String),
    #[error("Invalid master secret: {0}")]
    DuplicationMasterSecret(String),
    #[error("Unimplemented feature: {0}")]
    UnimplementedFeature(String),
}
