pub mod error;

use core::fmt;
use std::fmt::Display;

use did_parser::Did;
use error::DidKeyError;
use public_key::Key;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq)]
pub struct DidKey {
    key: Key,
    did: Did,
}

impl DidKey {
    pub fn parse<T>(did: T) -> Result<DidKey, DidKeyError>
    where
        Did: TryFrom<T>,
        <Did as TryFrom<T>>::Error: Into<DidKeyError>,
    {
        let did: Did = did.try_into().map_err(Into::into)?;
        let key = Key::from_fingerprint(did.id())?;

        Ok(Self { key, did })
    }

    pub fn key(&self) -> &Key {
        &self.key
    }

    pub fn did(&self) -> &Did {
        &self.did
    }
}

impl TryFrom<Key> for DidKey {
    type Error = DidKeyError;

    fn try_from(key: Key) -> Result<Self, Self::Error> {
        let did = Did::parse(format!("did:key:{}", key.fingerprint()))?;
        Ok(Self { key, did })
    }
}

impl Display for DidKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.did)
    }
}

impl Serialize for DidKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.did.did())
    }
}

impl<'de> Deserialize<'de> for DidKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        DidKey::parse(s).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn _valid_key_base58_fingerprint() -> String {
        "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string()
    }

    fn _valid_did_key_string() -> String {
        format!("did:key:{}", _valid_key_base58_fingerprint())
    }

    fn _invalid_did_key_string() -> String {
        "did:key:somenonsense".to_string()
    }

    fn _valid_did_key() -> DidKey {
        DidKey {
            key: Key::from_fingerprint(&_valid_key_base58_fingerprint()).unwrap(),
            did: Did::parse(_valid_did_key_string()).unwrap(),
        }
    }

    #[test]
    fn test_serialize() {
        assert_eq!(
            format!("\"{}\"", _valid_did_key_string()),
            serde_json::to_string(&_valid_did_key()).unwrap(),
        );
    }

    #[test]
    fn test_deserialize() {
        assert_eq!(
            _valid_did_key(),
            serde_json::from_str::<DidKey>(&format!("\"{}\"", _valid_did_key_string())).unwrap(),
        );
    }

    #[test]
    fn test_deserialize_error() {
        assert!(serde_json::from_str::<DidKey>(&_invalid_did_key_string()).is_err());
    }

    #[test]
    fn test_parse() {
        assert_eq!(_valid_did_key(), DidKey::parse(_valid_did_key_string()).unwrap(),);
    }

    #[test]
    fn test_parse_error() {
        assert!(DidKey::parse(_invalid_did_key_string()).is_err());
    }

    #[test]
    fn test_try_from_key() {
        assert_eq!(
            _valid_did_key(),
            DidKey::try_from(Key::from_fingerprint(&_valid_key_base58_fingerprint()).unwrap()).unwrap(),
        );
    }
}
