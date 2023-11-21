use thiserror::Error;

#[derive(Debug, Error)]
pub enum LedgerResponseParserError {
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::error::Error),
    #[error("Ledger item not found: {0}")]
    LedgerItemNotFound(&'static str),
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
}
