use aries_askar::{
    entry::Entry,
    kms::{KeyAlg, LocalKey},
};
use public_key::{Key, KeyType};
use serde::Deserialize;

use crate::{
    errors::error::VcxWalletResult,
    wallet::{base_wallet::base58_string::Base58String, utils::random_seed},
};

pub fn local_key_to_bs58_public_key(local_key: &LocalKey) -> VcxWalletResult<Base58String> {
    Ok(Base58String::from_bytes(&local_key.to_public_bytes()?))
}

pub fn local_key_to_bs58_private_key(local_key: &LocalKey) -> VcxWalletResult<Base58String> {
    Ok(Base58String::from_bytes(&local_key.to_secret_bytes()?))
}

pub fn local_key_to_public_key(local_key: &LocalKey) -> VcxWalletResult<Key> {
    Ok(Key::new(
        local_key.to_public_bytes()?.to_vec(),
        KeyType::Ed25519,
    )?)
}

pub fn ed25519_to_x25519(local_key: &LocalKey) -> VcxWalletResult<LocalKey> {
    Ok(local_key.convert_key(KeyAlg::X25519)?)
}

pub fn seed_from_opt(maybe_seed: Option<&str>) -> String {
    match maybe_seed {
        Some(val) => val.into(),
        None => random_seed(),
    }
}

pub fn from_json_str<T: for<'a> Deserialize<'a>>(json: &str) -> VcxWalletResult<T> {
    Ok(serde_json::from_str::<T>(json)?)
}

pub fn value_from_entry(entry: Entry) -> VcxWalletResult<String> {
    Ok(String::from_utf8(entry.value.to_vec())?)
}
