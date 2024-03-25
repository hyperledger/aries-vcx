use serde::{Deserialize, Serialize};

use crate::{
    errors::error::VcxWalletResult,
    wallet::utils::{bs58_to_bytes, bytes_to_bs58, bytes_to_string},
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct Base58String(String);

impl Base58String {
    pub fn from_bytes(content: &[u8]) -> Self {
        Self(bytes_to_bs58(content))
    }

    pub fn decode(&self) -> VcxWalletResult<Vec<u8>> {
        bs58_to_bytes(self.0.as_bytes())
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
