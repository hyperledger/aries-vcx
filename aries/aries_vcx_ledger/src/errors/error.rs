use aries_vcx_wallet::errors::error::VcxWalletError;
use did_parser_nom::ParseError;
use indy_vdr::common::error::VdrError;
use thiserror::Error as ThisError;

pub type VcxLedgerResult<T> = Result<T, VcxLedgerError>;

#[derive(Debug, ThisError)]
pub enum VcxLedgerError {
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(#[source] VdrError),
    #[error("Pool ledger connect: {0}")]
    PoolLedgerConnect(#[source] VdrError),
    #[error("IO error: {0}")]
    IOError(#[source] VdrError),
    #[error("Ledger item not found")]
    LedgerItemNotFound,
    #[error("Invalid ledger response {0}")]
    InvalidLedgerResponse(String),
    #[error("Duplicated schema")]
    DuplicationSchema,
    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("Wallet error: {0}")]
    WalletError(#[from] VcxWalletError),
    #[error("Invalid state: {0}")]
    InvalidState(String),
    #[error("Invalid option: {0}")]
    InvalidOption(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Unsupported ledger identifier: {0}")]
    UnsupportedLedgerIdentifier(String),
    #[error("Unknown error: {0}")]
    UnknownError(String),
    #[error("Indy Vdr Validation error: {0}")]
    IndyVdrValidation(#[source] indy_vdr::utils::ValidationError),
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
    #[error("Unimplemented feature: {0}")]
    UnimplementedFeature(String),
}
