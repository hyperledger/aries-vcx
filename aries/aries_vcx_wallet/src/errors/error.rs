use std::{fmt, string::FromUtf8Error};

use indy_vdr::utils::ConversionError;
use thiserror::Error as ThisError;
use vdrtools::IndyError;

use crate::wallet::base_wallet::{record_category::RecordCategory, search_filter::SearchFilter};

pub type VcxWalletResult<T> = Result<T, VcxWalletError>;

pub struct NotFoundInfo(Option<(RecordCategory, String)>);

impl NotFoundInfo {
    pub fn new(category: RecordCategory, name: &str) -> Self {
        Self(Some((category, name.to_string())))
    }

    pub fn new_without_details() -> Self {
        Self(None)
    }
}

impl fmt::Debug for NotFoundInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            None => write!(f, "no details provided"),
            Some(payload) => write!(f, "category: {}, name: {}", payload.0, payload.1),
        }
    }
}

impl fmt::Display for NotFoundInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, ThisError)]
pub enum VcxWalletError {
    #[error("Duplicate record error: {0}")]
    DuplicateRecord(String),
    #[error("Unexpected UTF-8 error: {0}")]
    NotUtf8(#[from] FromUtf8Error),
    #[error("String is not base58: {0}")]
    NotBase58(#[from] bs58::decode::Error),
    #[error("String is not base64: {0}")]
    NotBase64(#[from] ConversionError),
    #[error("Could not find record in wallet: {0}")]
    RecordNotFound(NotFoundInfo),
    #[error("Unknown record category: {0}")]
    UnknownRecordCategory(String),
    #[error("Not supported as filter type: {0}")]
    FilterTypeNotsupported(SearchFilter),
    #[error("Indy API error: {0}")]
    IndyApiError(#[from] IndyError),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("No recipient key found")]
    NoRecipientKeyFound,
    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("Public key error: {0}")]
    PublicKeyError(#[from] public_key::PublicKeyError),
    #[error("Unimplemented: {0}")]
    Unimplemented(String),
    #[error("Unknown error: {0}")]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
    #[error("Error when creating a wallet: {0}")]
    WalletCreate(Box<dyn std::error::Error + Send + Sync>),
}

impl VcxWalletError {
    pub fn create_wallet_error(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::WalletCreate(Box::new(err))
    }

    pub fn unknown_error(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Unknown(Box::new(err))
    }

    pub fn record_not_found(category: RecordCategory, name: &str) -> Self {
        Self::RecordNotFound(NotFoundInfo::new(category, name))
    }
}
