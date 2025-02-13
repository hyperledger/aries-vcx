use rand::{distr::Alphanumeric, Rng};

use crate::errors::error::VcxWalletResult;

#[allow(dead_code)]
pub fn random_seed() -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

pub fn bytes_to_string(vec: Vec<u8>) -> VcxWalletResult<String> {
    Ok(String::from_utf8(vec)?)
}

pub fn bytes_to_bs58(bytes: &[u8]) -> String {
    bs58::encode(bytes).into_string()
}

pub fn bs58_to_bytes(key: &[u8]) -> VcxWalletResult<Vec<u8>> {
    Ok(bs58::decode(key).into_vec()?)
}
