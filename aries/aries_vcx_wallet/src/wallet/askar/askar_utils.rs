use aries_askar::{
    crypto::alg::{BlsCurves, EcCurves, KeyAlg},
    entry::Entry,
    kms::LocalKey,
};
use public_key::{Key, KeyType};
use serde::Deserialize;

use crate::{
    errors::error::{VcxWalletError, VcxWalletResult},
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

pub fn public_key_to_local_key(key: &Key) -> VcxWalletResult<LocalKey> {
    let alg = public_key_type_to_askar_key_alg(key.key_type())?;
    Ok(LocalKey::from_public_bytes(alg, key.key())?)
}

pub fn public_key_type_to_askar_key_alg(value: &KeyType) -> VcxWalletResult<KeyAlg> {
    let alg = match value {
        KeyType::Ed25519 => KeyAlg::Ed25519,
        KeyType::X25519 => KeyAlg::X25519,
        KeyType::Bls12381g1g2 => KeyAlg::Bls12_381(BlsCurves::G1G2),
        KeyType::Bls12381g1 => KeyAlg::Bls12_381(BlsCurves::G1),
        KeyType::Bls12381g2 => KeyAlg::Bls12_381(BlsCurves::G2),
        KeyType::P256 => KeyAlg::EcCurve(EcCurves::Secp256r1),
        KeyType::P384 => KeyAlg::EcCurve(EcCurves::Secp384r1),
        _ => {
            return Err(VcxWalletError::Unimplemented(format!(
                "Unsupported key type: {value:?}"
            )))
        }
    };
    Ok(alg)
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
