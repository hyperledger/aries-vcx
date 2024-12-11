use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::KeyType;
use crate::error::PublicKeyError;

/// Represents raw public key data along with information about the key type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Key {
    key_type: KeyType,
    key: Vec<u8>,
}

impl Key {
    pub fn new(key: Vec<u8>, key_type: KeyType) -> Result<Self, PublicKeyError> {
        // If the key is a multibase key coming from a verification method, for some reason it is
        // also multicodec encoded, so we need to strip that. But should it be?
        let key = Self::strip_multicodec_prefix_if_present(key, &key_type);
        Ok(Self { key_type, key })
    }

    pub fn key_type(&self) -> &KeyType {
        &self.key_type
    }

    pub fn validate_key_type(&self, key_type: KeyType) -> Result<&Self, PublicKeyError> {
        if self.key_type() != &key_type {
            return Err(PublicKeyError::InvalidKeyType(
                self.key_type().to_owned(),
                key_type,
            ));
        }
        Ok(self)
    }

    pub fn key(&self) -> &[u8] {
        self.key.as_ref()
    }

    pub fn multicodec_prefixed_key(&self) -> Vec<u8> {
        let code = self.key_type().into();
        let mut buffer = [0u8; 10];
        let bytes = unsigned_varint::encode::u64(code, &mut buffer);
        let mut prefixed_key = bytes.to_vec();
        prefixed_key.extend_from_slice(&self.key);
        prefixed_key
    }

    pub fn fingerprint(&self) -> String {
        multibase::encode(multibase::Base::Base58Btc, self.multicodec_prefixed_key())
    }

    pub fn prefixless_fingerprint(&self) -> String {
        self.fingerprint().trim_start_matches('z').to_string()
    }

    pub fn base58(&self) -> String {
        bs58::encode(&self.key).into_string()
    }

    pub fn multibase58(&self) -> String {
        multibase::encode(multibase::Base::Base58Btc, &self.key)
    }

    pub fn from_fingerprint(fingerprint: &str) -> Result<Self, PublicKeyError> {
        let (_base, decoded_bytes) = multibase::decode(fingerprint)?;
        let (code, remaining_bytes) = unsigned_varint::decode::u64(&decoded_bytes)?;
        Ok(Self {
            key_type: code.try_into()?,
            key: remaining_bytes.to_vec(),
        })
    }

    pub fn from_base58(base58: &str, key_type: KeyType) -> Result<Self, PublicKeyError> {
        let decoded_bytes = bs58::decode(base58).into_vec()?;
        Ok(Self {
            key_type,
            key: decoded_bytes,
        })
    }

    pub fn short_prefixless_fingerprint(&self) -> String {
        self.prefixless_fingerprint()
            .chars()
            .take(8)
            .collect::<String>()
    }

    fn strip_multicodec_prefix_if_present(key: Vec<u8>, key_type: &KeyType) -> Vec<u8> {
        if let Ok((value, remaining)) = unsigned_varint::decode::u64(&key) {
            if value == Into::<u64>::into(key_type) {
                remaining.to_vec()
            } else {
                key
            }
        } else {
            key
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.base58())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_key_test(key_bytes: Vec<u8>, key_type: KeyType) {
        let key = Key::new(key_bytes.clone(), key_type);
        assert!(key.is_ok());
        let key = key.unwrap();
        assert_eq!(key.key_type(), &key_type);
        assert_eq!(key.key(), key_bytes.as_slice());
    }

    fn prefixed_key_test(
        key_bytes: Vec<u8>,
        key_type: KeyType,
        expected_prefixed_key: String,
        encode_fn: fn(Vec<u8>) -> String,
    ) {
        let key = Key::new(key_bytes, key_type).unwrap();
        let prefixed_key = key.multicodec_prefixed_key();
        assert_eq!(expected_prefixed_key, encode_fn(prefixed_key),);
    }

    fn fingerprint_test(key_bytes: Vec<u8>, key_type: KeyType, expected_fingerprint: &str) {
        let key = Key::new(key_bytes, key_type).unwrap();
        let fingerprint = key.fingerprint();
        assert_eq!(fingerprint, expected_fingerprint);
    }

    fn base58_test(key_bytes: Vec<u8>, key_type: KeyType, expected_base58: &str) {
        let key = Key::new(key_bytes, key_type).unwrap();
        let base58 = key.base58();
        assert_eq!(base58, expected_base58);
    }

    fn from_fingerprint_test(key_bytes: Vec<u8>, key_type: KeyType, fingerprint: &str) {
        let key = Key::new(key_bytes, key_type).unwrap();
        let key_from_fingerprint = Key::from_fingerprint(fingerprint);
        assert!(key_from_fingerprint.is_ok());
        let key_from_fingerprint = key_from_fingerprint.unwrap();
        assert_eq!(key.key_type(), key_from_fingerprint.key_type());
        assert_eq!(key.key(), key_from_fingerprint.key());
        assert_eq!(
            key.multicodec_prefixed_key(),
            key_from_fingerprint.multicodec_prefixed_key()
        );
        assert_eq!(key.fingerprint(), fingerprint);
        assert_eq!(key_from_fingerprint.fingerprint(), fingerprint);
    }

    fn strip_multicodec_prefix_if_present_test(key_bytes: Vec<u8>, key_type: &KeyType) {
        let key_type_u64: u64 = key_type.into();

        let mut buffer = [0u8; 10];
        let key_type_bytes = unsigned_varint::encode::u64(key_type_u64, &mut buffer);
        let mut prefixed_key = key_type_bytes.to_vec();
        prefixed_key.extend_from_slice(&key_bytes);

        let (_, remaining_bytes) = unsigned_varint::decode::u64(&prefixed_key).unwrap();
        let manually_stripped_key = remaining_bytes.to_vec();

        let function_stripped_key = Key::strip_multicodec_prefix_if_present(prefixed_key, key_type);
        assert_eq!(function_stripped_key, manually_stripped_key);

        let no_prefix_stripped_key =
            Key::strip_multicodec_prefix_if_present(key_bytes.clone(), key_type);
        assert_eq!(no_prefix_stripped_key, key_bytes);
    }

    #[test]
    fn from_fingerprint_error_test() {
        let key = Key::from_fingerprint("this is not a valid fingerprint");
        assert!(key.is_err());
    }

    mod ed25519 {
        use super::*;

        const TEST_KEY_BASE58: &str = "8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K";
        const TEST_FINGERPRINT: &str = "z6MkmjY8GnV5i9YTDtPETC2uUAW6ejw3nk5mXF5yci5ab7th";

        fn key_bytes() -> Vec<u8> {
            bs58::decode(TEST_KEY_BASE58).into_vec().unwrap()
        }

        fn encode_multibase(key_bytes: Vec<u8>) -> String {
            multibase::encode(multibase::Base::Base58Btc, key_bytes)
        }

        #[test]
        fn new_key_test() {
            super::new_key_test(key_bytes(), KeyType::Ed25519);
        }

        #[test]
        fn prefixed_key_test() {
            super::prefixed_key_test(
                key_bytes(),
                KeyType::Ed25519,
                TEST_FINGERPRINT.to_string(),
                encode_multibase,
            );
        }

        #[test]
        fn fingerprint_test() {
            super::fingerprint_test(key_bytes(), KeyType::Ed25519, TEST_FINGERPRINT);
        }

        #[test]
        fn base58_test() {
            super::base58_test(key_bytes(), KeyType::Ed25519, TEST_KEY_BASE58);
        }

        #[test]
        fn from_fingerprint_test() {
            super::from_fingerprint_test(key_bytes(), KeyType::Ed25519, TEST_FINGERPRINT);
        }

        #[test]
        fn strip_multicodec_prefix_if_present_test() {
            super::strip_multicodec_prefix_if_present_test(key_bytes(), &KeyType::Ed25519);
        }
    }

    mod x25519 {
        use super::*;

        const TEST_KEY_BASE58: &str = "6fUMuABnqSDsaGKojbUF3P7ZkEL3wi2njsDdUWZGNgCU";
        const TEST_FINGERPRINT: &str = "z6LShLeXRTzevtwcfehaGEzCMyL3bNsAeKCwcqwJxyCo63yE";

        fn key_bytes() -> Vec<u8> {
            bs58::decode(TEST_KEY_BASE58).into_vec().unwrap()
        }

        fn encode_multibase(key_bytes: Vec<u8>) -> String {
            multibase::encode(multibase::Base::Base58Btc, key_bytes)
        }

        #[test]
        fn new_key_test() {
            super::new_key_test(key_bytes(), KeyType::X25519);
        }

        #[test]
        fn prefixed_key_test() {
            super::prefixed_key_test(
                key_bytes(),
                KeyType::X25519,
                TEST_FINGERPRINT.to_string(),
                encode_multibase,
            );
        }

        #[test]
        fn fingerprint_test() {
            super::fingerprint_test(key_bytes(), KeyType::X25519, TEST_FINGERPRINT);
        }

        #[test]
        fn base58_test() {
            super::base58_test(key_bytes(), KeyType::X25519, TEST_KEY_BASE58);
        }

        #[test]
        fn from_fingerprint_test() {
            super::from_fingerprint_test(key_bytes(), KeyType::X25519, TEST_FINGERPRINT);
        }

        #[test]
        fn strip_multicodec_prefix_if_present_test() {
            super::strip_multicodec_prefix_if_present_test(key_bytes(), &KeyType::X25519);
        }
    }
}
