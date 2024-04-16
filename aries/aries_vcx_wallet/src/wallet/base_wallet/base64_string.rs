use indy_vdr::utils::base64::{decode_urlsafe, encode_urlsafe};
use serde::{Deserialize, Serialize};

use crate::{errors::error::VcxWalletResult, wallet::utils::bytes_to_string};

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct Base64String(String);

impl Base64String {
    pub fn from_bytes(content: &[u8]) -> Self {
        Self(encode_urlsafe(content))
    }

    pub fn decode(&self) -> VcxWalletResult<Vec<u8>> {
        Ok(decode_urlsafe(&self.0)?)
    }

    pub fn decode_to_string(&self) -> VcxWalletResult<String> {
        bytes_to_string(self.decode()?)
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().into()
    }
}
