use base64::{
    alphabet,
    engine::{general_purpose, DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig},
    Engine,
};
use serde::Deserialize;

use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult};

const ANY_PADDING: GeneralPurposeConfig =
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent);
const URL_SAFE_ANY_PADDING: GeneralPurpose = GeneralPurpose::new(&alphabet::URL_SAFE, ANY_PADDING);

pub fn encode_urlsafe(content: &[u8]) -> String {
    general_purpose::URL_SAFE.encode(content)
}

pub fn decode_urlsafe(content: &str) -> VcxCoreResult<Vec<u8>> {
    URL_SAFE_ANY_PADDING
        .decode(content)
        .map_err(|e| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidJson, e))
}

pub fn bytes_to_string(vec: Vec<u8>) -> VcxCoreResult<String> {
    Ok(String::from_utf8(vec)
        .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err))?)
}

pub fn bs58_to_bytes(key: &str) -> VcxCoreResult<Vec<u8>> {
    Ok(bs58::decode(key)
        .into_vec()
        .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::WalletUnexpected, err))?)
}

pub fn bytes_to_bs58(bytes: &[u8]) -> String {
    bs58::encode(bytes).into_string()
}

pub fn from_json_str<T: for<'a> Deserialize<'a>>(json: &str) -> VcxCoreResult<T> {
    Ok(serde_json::from_str::<T>(json)
        .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidJson, err))?)
}
