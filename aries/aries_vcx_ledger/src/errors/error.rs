use aries_vcx_wallet::errors::error::VcxWalletError;
use did_parser_nom::ParseError;
use indy_vdr::common::error::VdrError;
use thiserror::Error as ThisError;

pub type VcxLedgerResult<T> = Result<T, VcxLedgerError>;

#[derive(Debug, ThisError)]
pub enum VcxLedgerError {
    #[error("Ledger item not found")]
    LedgerItemNotFound,
    #[error("Invalid ledger response")]
    InvalidLedgerResponse,
    #[error("Duplicated schema")]
    DuplicationSchema,
    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("Vdr error: {0}")]
    VdrError(#[from] VdrError),
    #[error("Wallet error: {0}")]
    WalletError(#[from] VcxWalletError),
    #[error("Invalid state: {0}")]
    InvalidState(String),
    #[error("Invalid option: {0}")]
    InvalidOption(String),
    #[error("Indy Vdr Validation error: {0}")]
    IndyVdrValidation(#[from] indy_vdr::utils::ValidationError),
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
    #[error("Unimplemented feature: {0}")]
    UnimplementedFeature(String),
}
