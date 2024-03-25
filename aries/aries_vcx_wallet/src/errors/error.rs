use std::{
    fmt::{self, Display},
    string::FromUtf8Error,
};

use indy_vdr::utils::ConversionError;
use thiserror::Error as ThisError;

use crate::wallet::base_wallet::{record_category::RecordCategory, search_filter::SearchFilter};

pub type VcxWalletResult<T> = Result<T, VcxWalletError>;

#[derive(Debug, ThisError)]
pub enum VcxWalletError {
    #[error("Duplicate record: {0}")]
    DuplicateRecord(String),
    #[error("Not UTF-8: {0}")]
    NotUtf8(FromUtf8Error),
    #[error("Not Base58: {0}")]
    NotBase58(bs58::decode::Error),
    #[error("Not Base64: {0}")]
    NotBase64(ConversionError),
    #[error("Record not found: {0}")]
    RecordNotFound(String),
    #[error("Unknown record category: {0}")]
    UnknownRecordCategory(String),
    #[error("Filter type not supported: {0}")]
    FilterTypeNotsupported(SearchFilter),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Invalid WQL: {0}")]
    InvalidWql(String),
    #[error("No recipient found")]
    NoRecipientKeyFound,
    #[error("Invalid JSON: {0}")]
    InvalidJson(serde_json::Error),
    #[error("Public key error: {0}")]
    PublicKeyError(public_key::PublicKeyError),
    #[error("Unimplemented: {0}")]
    Unimplemented(String),
    #[error("Unknown error: {0}")]
    Unknown(OpaqueError),
    #[error("Error when creating a wallet: {0}")]
    WalletCreate(OpaqueError),
}

#[derive(ThisError, Debug)]
pub struct OpaqueError(anyhow::Error);

impl Display for OpaqueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl VcxWalletError {
    pub fn create_wallet_error(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::WalletCreate(OpaqueError(err.into()))
    }

    pub fn unknown_error(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Unknown(OpaqueError(err.into()))
    }

    pub fn record_not_found(category: RecordCategory, name: &str) -> Self {
        Self::RecordNotFound(format!("category: {}, name: {}", category, name))
    }
}
