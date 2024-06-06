use base64::{
    alphabet,
    engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig},
    Engine,
};
use serde::{Deserialize, Serialize};

use crate::{errors::error::VcxWalletResult, wallet::utils::bytes_to_string};

/// A default [GeneralPurposeConfig] configuration with a [decode_padding_mode] of
/// [DecodePaddingMode::Indifferent]
const LENIENT_PAD: GeneralPurposeConfig = GeneralPurposeConfig::new()
    .with_encode_padding(false)
    .with_decode_padding_mode(DecodePaddingMode::Indifferent);

/// A [GeneralPurpose] engine using the [alphabet::URL_SAFE] base64 alphabet and
/// [DecodePaddingMode::Indifferent] config to decode both padded and unpadded.
const URL_SAFE_LENIENT: GeneralPurpose = GeneralPurpose::new(&alphabet::URL_SAFE, LENIENT_PAD);

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct Base64String(String);

impl Base64String {
    pub fn from_bytes(content: &[u8]) -> Self {
        Self(URL_SAFE_LENIENT.encode(content))
    }

    pub fn decode(&self) -> VcxWalletResult<Vec<u8>> {
        Ok(URL_SAFE_LENIENT.decode(&self.0)?)
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
