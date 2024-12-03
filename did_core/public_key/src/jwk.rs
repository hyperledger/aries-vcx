use askar_crypto::{
    alg::{AnyKey, AnyKeyCreate, BlsCurves, EcCurves, KeyAlg},
    jwk::{FromJwk, ToJwk},
    repr::ToPublicBytes,
};

use crate::{Key, KeyType, PublicKeyError};

impl Key {
    pub fn from_jwk(jwk: &str) -> Result<Key, PublicKeyError> {
        let askar_key: Box<AnyKey> =
            FromJwk::from_jwk(jwk).map_err(|e| PublicKeyError::JwkDecodingError(Box::new(e)))?;

        let askar_alg = askar_key.algorithm();
        let pub_key_bytes = askar_key
            .to_public_bytes()
            .map_err(|e| PublicKeyError::JwkDecodingError(Box::new(e)))?
            .to_vec();

        let key_type = match askar_alg {
            askar_crypto::alg::KeyAlg::Ed25519 => KeyType::Ed25519,
            askar_crypto::alg::KeyAlg::Bls12_381(BlsCurves::G1G2) => KeyType::Bls12381g1g2,
            askar_crypto::alg::KeyAlg::Bls12_381(BlsCurves::G1) => KeyType::Bls12381g1,
            askar_crypto::alg::KeyAlg::Bls12_381(BlsCurves::G2) => KeyType::Bls12381g2,
            askar_crypto::alg::KeyAlg::X25519 => KeyType::X25519,
            askar_crypto::alg::KeyAlg::EcCurve(EcCurves::Secp256r1) => KeyType::P256,
            askar_crypto::alg::KeyAlg::EcCurve(EcCurves::Secp384r1) => KeyType::P384,
            _ => return Err(PublicKeyError::UnsupportedKeyType(askar_alg.to_string())),
        };

        Key::new(pub_key_bytes, key_type)
    }

    pub fn to_jwk(&self) -> Result<String, PublicKeyError> {
        let askar_key = self.to_askar_local_key()?;
        askar_key.to_jwk_public(None).map_err(|e| {
            PublicKeyError::UnsupportedKeyType(format!("Could not process this key as JWK {e:?}"))
        })
    }

    fn to_askar_local_key(&self) -> Result<Box<AnyKey>, PublicKeyError> {
        let alg = public_key_type_to_askar_key_alg(self.key_type())?;
        AnyKeyCreate::from_public_bytes(alg, self.key()).map_err(|e| {
            PublicKeyError::UnsupportedKeyType(format!("Could not process key type {alg:?}: {e:?}"))
        })
    }
}

pub fn public_key_type_to_askar_key_alg(value: &KeyType) -> Result<KeyAlg, PublicKeyError> {
    let alg = match value {
        KeyType::Ed25519 => KeyAlg::Ed25519,
        KeyType::X25519 => KeyAlg::X25519,
        KeyType::Bls12381g1g2 => KeyAlg::Bls12_381(BlsCurves::G1G2),
        KeyType::Bls12381g1 => KeyAlg::Bls12_381(BlsCurves::G1),
        KeyType::Bls12381g2 => KeyAlg::Bls12_381(BlsCurves::G2),
        KeyType::P256 => KeyAlg::EcCurve(EcCurves::Secp256r1),
        KeyType::P384 => KeyAlg::EcCurve(EcCurves::Secp384r1),
        other => {
            return Err(PublicKeyError::UnsupportedKeyType(format!(
                "Unsupported key type: {other:?}"
            )));
        }
    };
    Ok(alg)
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    // vector from https://w3c-ccg.github.io/did-method-key/#ed25519-x25519
    const ED25519_JWK: &str = r#"{
        "kty": "OKP",
        "crv": "Ed25519",
        "x": "O2onvM62pC1io6jQKm8Nc2UyFXcd4kOmOsBIoYtZ2ik"
    }"#;
    const ED25519_FINGERPRINT: &str = "z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp";

    // vector from https://w3c-ccg.github.io/did-method-key/#ed25519-x25519
    const X25519_JWK: &str = r#"{
        "kty": "OKP",
        "crv": "X25519",
        "x": "W_Vcc7guviK-gPNDBmevVw-uJVamQV5rMNQGUwCqlH0"
    }"#;
    const X25519_FINGERPRINT: &str = "z6LShs9GGnqk85isEBzzshkuVWrVKsRp24GnDuHk8QWkARMW";

    // vector from https://dev.uniresolver.io/
    const P256_JWK: &str = r#"{
        "kty": "EC",
        "crv": "P-256",
        "x": "fyNYMN0976ci7xqiSdag3buk-ZCwgXU4kz9XNkBlNUI",
        "y": "hW2ojTNfH7Jbi8--CJUo3OCbH3y5n91g-IMA9MLMbTU"
    }"#;
    const P256_FINGERPRINT: &str = "zDnaerDaTF5BXEavCrfRZEk316dpbLsfPDZ3WJ5hRTPFU2169";

    // vector from https://dev.uniresolver.io/
    const P384_JWK: &str = r#"{
        "kty": "EC",
        "crv": "P-384",
        "x": "bKq-gg3sJmfkJGrLl93bsumOTX1NubBySttAV19y5ClWK3DxEmqPy0at5lLqBiiv",
        "y": "PJQtdHnInU9SY3e8Nn9aOPoP51OFbs-FWJUsU0TGjRtZ4bnhoZXtS92wdzuAotL9"
    }"#;
    const P384_FINGERPRINT: &str =
        "z82Lkytz3HqpWiBmt2853ZgNgNG8qVoUJnyoMvGw6ZEBktGcwUVdKpUNJHct1wvp9pXjr7Y";

    #[test]
    fn test_from_ed25519_jwk() {
        let key = Key::from_jwk(ED25519_JWK).unwrap();
        assert_eq!(key.fingerprint(), ED25519_FINGERPRINT);
    }

    #[test]
    fn test_from_x25519_jwk() {
        let key = Key::from_jwk(X25519_JWK).unwrap();
        assert_eq!(key.fingerprint(), X25519_FINGERPRINT);
    }

    #[test]
    fn test_from_p256_jwk() {
        let key = Key::from_jwk(P256_JWK).unwrap();
        assert_eq!(key.fingerprint(), P256_FINGERPRINT);
    }

    #[test]
    fn test_from_p384_jwk() {
        let key = Key::from_jwk(P384_JWK).unwrap();
        assert_eq!(key.fingerprint(), P384_FINGERPRINT);
    }

    #[test]
    fn test_ed25519_to_jwk() {
        let key = Key::from_fingerprint(ED25519_FINGERPRINT).unwrap();
        let jwk: Value = serde_json::from_str(&key.to_jwk().unwrap()).unwrap();
        assert_eq!(jwk, serde_json::from_str::<Value>(ED25519_JWK).unwrap());
    }

    #[test]
    fn test_x25519_to_jwk() {
        let key = Key::from_fingerprint(X25519_FINGERPRINT).unwrap();
        let jwk: Value = serde_json::from_str(&key.to_jwk().unwrap()).unwrap();
        assert_eq!(jwk, serde_json::from_str::<Value>(X25519_JWK).unwrap());
    }

    #[test]
    fn test_p256_to_jwk() {
        let key = Key::from_fingerprint(P256_FINGERPRINT).unwrap();
        let jwk: Value = serde_json::from_str(&key.to_jwk().unwrap()).unwrap();
        assert_eq!(jwk, serde_json::from_str::<Value>(P256_JWK).unwrap());
    }

    #[test]
    fn test_p384_to_jwk() {
        let key = Key::from_fingerprint(P384_FINGERPRINT).unwrap();
        let jwk: Value = serde_json::from_str(&key.to_jwk().unwrap()).unwrap();
        assert_eq!(jwk, serde_json::from_str::<Value>(P384_JWK).unwrap());
    }
}
