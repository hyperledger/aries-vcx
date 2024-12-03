use std::fmt::{self, Display};

use base64::{
    alphabet,
    engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig},
    Engine,
};
use did_doc::schema::types::jsonwebkey::JsonWebKey;
use did_parser_nom::Did;
use error::DidJwkError;
use public_key::{Key, KeyType};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::json;

pub mod error;
pub mod resolver;

const USE: &str = "use";
const USE_SIG: &str = "sig";
const USE_ENC: &str = "enc";

/// A default [GeneralPurposeConfig] configuration with a [decode_padding_mode] of
/// [DecodePaddingMode::Indifferent]
const LENIENT_PAD: GeneralPurposeConfig = GeneralPurposeConfig::new()
    .with_encode_padding(false)
    .with_decode_padding_mode(DecodePaddingMode::Indifferent);

/// A [GeneralPurpose] engine using the [alphabet::URL_SAFE] base64 alphabet and
/// [DecodePaddingMode::Indifferent] config to decode both padded and unpadded.
const URL_SAFE_LENIENT: GeneralPurpose = GeneralPurpose::new(&alphabet::URL_SAFE, LENIENT_PAD);

/// Represents did:key where the DID ID is JWK public key itself
/// See the spec: https://github.com/quartzjer/did-jwk/blob/main/spec.md
#[derive(Clone, Debug, PartialEq)]
pub struct DidJwk {
    jwk: JsonWebKey,
    did: Did,
}

impl DidJwk {
    pub fn parse<T>(did: T) -> Result<DidJwk, DidJwkError>
    where
        Did: TryFrom<T>,
        <Did as TryFrom<T>>::Error: Into<DidJwkError>,
    {
        let did: Did = did.try_into().map_err(Into::into)?;
        Self::try_from(did)
    }

    pub fn try_from_serialized_jwk(jwk: &str) -> Result<DidJwk, DidJwkError> {
        let jwk: JsonWebKey = serde_json::from_str(jwk)?;
        Self::try_from(jwk)
    }

    pub fn jwk(&self) -> &JsonWebKey {
        &self.jwk
    }

    pub fn did(&self) -> &Did {
        &self.did
    }

    pub fn key(&self) -> Result<Key, DidJwkError> {
        Ok(Key::from_jwk(&serde_json::to_string(&self.jwk)?)?)
    }
}

impl TryFrom<Did> for DidJwk {
    type Error = DidJwkError;

    fn try_from(did: Did) -> Result<Self, Self::Error> {
        match did.method() {
            Some("jwk") => {}
            other => return Err(DidJwkError::MethodNotSupported(format!("{other:?}"))),
        }

        let jwk = decode_jwk(did.id())?;
        Ok(Self { jwk, did })
    }
}

impl TryFrom<JsonWebKey> for DidJwk {
    type Error = DidJwkError;

    fn try_from(jwk: JsonWebKey) -> Result<Self, Self::Error> {
        let encoded_jwk = encode_jwk(&jwk)?;
        let did = Did::parse(format!("did:jwk:{encoded_jwk}",))?;

        Ok(Self { jwk, did })
    }
}

impl TryFrom<Key> for DidJwk {
    type Error = DidJwkError;

    fn try_from(key: Key) -> Result<Self, Self::Error> {
        let jwk = key.to_jwk()?;
        let mut jwk: JsonWebKey = serde_json::from_str(&jwk)?;

        match key.key_type() {
            KeyType::Ed25519
            | KeyType::Bls12381g1g2
            | KeyType::Bls12381g1
            | KeyType::Bls12381g2
            | KeyType::P256
            | KeyType::P384
            | KeyType::P521 => {
                jwk.extra.insert(String::from(USE), json!(USE_SIG));
            }
            KeyType::X25519 => {
                jwk.extra.insert(String::from(USE), json!(USE_ENC));
            }
        }

        Self::try_from(jwk)
    }
}

impl Display for DidJwk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.did)
    }
}

impl Serialize for DidJwk {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.did.did())
    }
}

impl<'de> Deserialize<'de> for DidJwk {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        DidJwk::parse(s).map_err(de::Error::custom)
    }
}

fn encode_jwk(jwk: &JsonWebKey) -> Result<String, DidJwkError> {
    let jwk_bytes = serde_json::to_vec(jwk)?;
    Ok(URL_SAFE_LENIENT.encode(jwk_bytes))
}

fn decode_jwk(encoded_jwk: &str) -> Result<JsonWebKey, DidJwkError> {
    let jwk_bytes = URL_SAFE_LENIENT.decode(encoded_jwk)?;
    Ok(serde_json::from_slice(&jwk_bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_key_base58_fingerprint() -> String {
        "z6MkeWVt6dndY6EbFwEvb3VQU6ksQXKTeorkQ9sU29DY7yRX".to_string()
    }

    fn valid_key() -> Key {
        Key::from_fingerprint(&valid_key_base58_fingerprint()).unwrap()
    }

    fn valid_serialized_jwk() -> String {
        r#"{
            "kty": "OKP",
            "crv": "Ed25519",
            "x": "ANRjH_zxcKBxsjRPUtzRbp7FSVLKJXQ9APX9MP1j7k4",
            "use": "sig"
        }"#
        .to_string()
    }

    fn valid_jwk() -> JsonWebKey {
        serde_json::from_str(&valid_serialized_jwk()).unwrap()
    }

    fn valid_encoded_jwk() -> String {
        URL_SAFE_LENIENT.encode(serde_json::to_vec(&valid_jwk()).unwrap())
    }

    fn valid_did_jwk_string() -> String {
        format!("did:jwk:{}", valid_encoded_jwk())
    }

    fn invalid_did_jwk_string_wrong_method() -> String {
        format!("did:sov:{}", valid_encoded_jwk())
    }

    fn invalid_did_jwk_string_invalid_id() -> String {
        "did:jwk:somenonsense".to_string()
    }

    fn valid_did_jwk() -> DidJwk {
        DidJwk {
            jwk: valid_jwk(),
            did: Did::parse(valid_did_jwk_string()).unwrap(),
        }
    }

    #[test]
    fn test_serialize() {
        assert_eq!(
            format!("\"{}\"", valid_did_jwk_string()),
            serde_json::to_string(&valid_did_jwk()).unwrap(),
        );
    }

    #[test]
    fn test_deserialize() {
        assert_eq!(
            valid_did_jwk(),
            serde_json::from_str::<DidJwk>(&format!("\"{}\"", valid_did_jwk_string())).unwrap(),
        );
    }

    #[test]
    fn test_deserialize_error_wrong_method() {
        assert!(serde_json::from_str::<DidJwk>(&invalid_did_jwk_string_wrong_method()).is_err());
    }

    #[test]
    fn test_deserialize_error_invalid_id() {
        assert!(serde_json::from_str::<DidJwk>(&invalid_did_jwk_string_invalid_id()).is_err());
    }

    #[test]
    fn test_parse() {
        assert_eq!(
            valid_did_jwk(),
            DidJwk::parse(valid_did_jwk_string()).unwrap(),
        );
    }

    #[test]
    fn test_parse_error_wrong_method() {
        assert!(DidJwk::parse(invalid_did_jwk_string_wrong_method()).is_err());
    }

    #[test]
    fn test_parse_error_invalid_id() {
        assert!(DidJwk::parse(invalid_did_jwk_string_invalid_id()).is_err());
    }

    #[test]
    fn test_to_key() {
        assert_eq!(valid_did_jwk().key().unwrap(), valid_key());
    }

    #[test]
    fn test_try_from_serialized_jwk() {
        assert_eq!(
            valid_did_jwk(),
            DidJwk::try_from_serialized_jwk(&valid_serialized_jwk()).unwrap(),
        );
    }

    #[test]
    fn test_try_from_jwk() {
        assert_eq!(valid_did_jwk(), DidJwk::try_from(valid_jwk()).unwrap(),);
    }

    #[test]
    fn test_try_from_key() {
        assert_eq!(valid_did_jwk(), DidJwk::try_from(valid_key()).unwrap(),);
    }
}
