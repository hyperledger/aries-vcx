use std::{
    fmt::{self, Display},
    string::FromUtf8Error,
};

use thiserror::Error as ThisError;

use crate::wallet::base_wallet::record_category::RecordCategory;

pub type VcxWalletResult<T> = Result<T, VcxWalletError>;

pub struct NotFoundInfo(Option<String>);

impl NotFoundInfo {
    pub fn new_with_details(category: RecordCategory, name: &str) -> Self {
        Self(Some(format!(
            "Not found, category: {}, name {}",
            category, name
        )))
    }

    pub fn new_from_str(info: &str) -> Self {
        Self(Some(info.to_string()))
    }

    pub fn new_without_details() -> Self {
        Self(None)
    }
}

impl fmt::Debug for NotFoundInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            None => write!(f, "no details provided"),
            Some(payload) => write!(f, "{}", payload),
        }
    }
}

impl fmt::Display for NotFoundInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug)]
pub enum VcxWalletError {
    DuplicateRecord(String),
    NotUtf8(FromUtf8Error),
    NotBase58(bs58::decode::Error),
    NotBase64(base64::DecodeError),
    RecordNotFound(NotFoundInfo),
    UnknownRecordCategory(String),
    InvalidInput(String),
    NoRecipientKeyFound,
    InvalidJson(serde_json::Error),
    PublicKeyError(public_key::PublicKeyError),
    Unimplemented(String),
    Unknown(OpaqueError),
    WalletCreate(OpaqueError),
}

#[derive(ThisError, Debug)]
pub struct OpaqueError(anyhow::Error);

impl Display for OpaqueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for VcxWalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VcxWalletError::DuplicateRecord(inner) => write!(f, "Duplicate record: {}", inner),
            VcxWalletError::NotUtf8(inner) => write!(f, "Not UTF-8: {}", inner),
            VcxWalletError::NotBase58(inner) => write!(f, "Not Base58: {}", inner),
            VcxWalletError::NotBase64(inner) => write!(f, "Not Base64: {}", inner),
            VcxWalletError::RecordNotFound(inner) => write!(f, "Record not found: {}", inner),
            VcxWalletError::UnknownRecordCategory(inner) => {
                write!(f, "Unknown RecordCategory: {}", inner)
            }
            VcxWalletError::InvalidInput(inner) => write!(f, "Invalid input: {}", inner),
            VcxWalletError::NoRecipientKeyFound => write!(f, "No recipient key found"),
            VcxWalletError::InvalidJson(inner) => write!(f, "Invalid JSON: {}", inner),
            VcxWalletError::PublicKeyError(inner) => write!(f, "Public key error: {}", inner),
            VcxWalletError::Unimplemented(inner) => write!(f, "Not implemented: {}", inner),
            VcxWalletError::Unknown(inner) => write!(f, "Unknown error: {}", inner),
            VcxWalletError::WalletCreate(inner) => write!(f, "Error creating a wallet: {}", inner),
        }
    }
}

impl std::error::Error for VcxWalletError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            VcxWalletError::DuplicateRecord(_) => None,
            VcxWalletError::NotUtf8(inner) => Some(inner),
            VcxWalletError::NotBase58(inner) => Some(inner),
            VcxWalletError::NotBase64(inner) => Some(inner),
            VcxWalletError::RecordNotFound(_) => None,
            VcxWalletError::UnknownRecordCategory(_) => None,
            VcxWalletError::InvalidInput(_) => None,
            VcxWalletError::NoRecipientKeyFound => None,
            VcxWalletError::InvalidJson(inner) => Some(inner),
            VcxWalletError::PublicKeyError(inner) => Some(inner),
            VcxWalletError::Unimplemented(_) => None,
            VcxWalletError::Unknown(inner) => Some(inner),
            VcxWalletError::WalletCreate(inner) => Some(inner),
        }
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl VcxWalletError {
    pub fn create_wallet_error(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::WalletCreate(OpaqueError(err.into()))
    }

    pub fn unknown_error(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Unknown(OpaqueError(err.into()))
    }

    pub fn record_not_found_from_details(category: RecordCategory, name: &str) -> Self {
        Self::RecordNotFound(NotFoundInfo::new_with_details(category, name))
    }

    pub fn record_not_found_from_str(info: &str) -> Self {
        Self::RecordNotFound(NotFoundInfo::new_from_str(info))
    }
}
