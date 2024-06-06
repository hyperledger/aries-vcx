use std::string::FromUtf8Error;


use super::error::VcxWalletError;

impl From<bs58::decode::Error> for VcxWalletError {
    fn from(value: bs58::decode::Error) -> Self {
        Self::NotBase58(value)
    }
}

impl From<base64::DecodeError> for VcxWalletError {
    fn from(value: base64::DecodeError) -> Self {
        Self::NotBase64(value)
    }
}

impl From<FromUtf8Error> for VcxWalletError {
    fn from(value: FromUtf8Error) -> Self {
        Self::NotUtf8(value)
    }
}

impl From<serde_json::Error> for VcxWalletError {
    fn from(value: serde_json::Error) -> Self {
        Self::InvalidJson(value)
    }
}

impl From<public_key::PublicKeyError> for VcxWalletError {
    fn from(value: public_key::PublicKeyError) -> Self {
        Self::PublicKeyError(value)
    }
}
