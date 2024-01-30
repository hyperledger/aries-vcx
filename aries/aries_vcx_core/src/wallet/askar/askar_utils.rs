use aries_askar::{
    entry::Entry,
    kms::{KeyAlg, LocalKey},
};
use public_key::{Key, KeyType};
use serde::Deserialize;

use crate::{
    errors::error::{AriesVcxCoreErrorKind, VcxCoreResult},
    wallet::{askar::AriesVcxCoreError, utils::random_seed},
};

pub fn local_key_to_bs58_name(local_key: &LocalKey) -> VcxCoreResult<String> {
    let res = local_key_to_bs58_public_key(local_key)?;
    Ok(res[0..16].to_string())
}

pub fn local_key_to_bs58_public_key(local_key: &LocalKey) -> VcxCoreResult<String> {
    Ok(bs58::encode(local_key.to_public_bytes()?).into_string())
}

pub fn local_key_to_bs58_private_key(local_key: &LocalKey) -> VcxCoreResult<String> {
    Ok(bs58::encode(local_key.to_secret_bytes()?).into_string())
}

pub fn local_key_to_public_key(local_key: &LocalKey) -> VcxCoreResult<Key> {
    Ok(Key::new(
        local_key.to_public_bytes()?.to_vec(),
        KeyType::Ed25519,
    )?)
}

pub fn ed25519_to_x25519(local_key: &LocalKey) -> VcxCoreResult<LocalKey> {
    Ok(local_key.convert_key(KeyAlg::X25519)?)
}

pub fn seed_from_opt(maybe_seed: Option<&str>) -> String {
    match maybe_seed {
        Some(val) => val.into(),
        None => random_seed(),
    }
}

pub fn from_json_str<T: for<'a> Deserialize<'a>>(json: &str) -> VcxCoreResult<T> {
    serde_json::from_str::<T>(json)
        .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidJson, err))
}

pub fn bytes_to_bs58(bytes: &[u8]) -> String {
    bs58::encode(bytes).into_string()
}

pub fn bs58_to_bytes(key: &[u8]) -> VcxCoreResult<Vec<u8>> {
    bs58::decode(key)
        .into_vec()
        .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::WalletError, err))
}

pub fn bytes_to_string(vec: Vec<u8>) -> VcxCoreResult<String> {
    String::from_utf8(vec)
        .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidInput, err))
}

pub fn value_from_entry(entry: Entry) -> VcxCoreResult<String> {
    Ok(std::str::from_utf8(&entry.value)
        .map_err(|err| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::WalletError, err))?
        .to_string())
}
