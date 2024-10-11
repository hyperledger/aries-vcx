use askar_crypto::{
    alg::{AnyKey, BlsCurves, EcCurves},
    jwk::FromJwk,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ed25519_jwk() {
        // vector from https://w3c-ccg.github.io/did-method-key/#ed25519-x25519
        let jwk = r#"{
            "kty": "OKP",
            "crv": "Ed25519",
            "x": "O2onvM62pC1io6jQKm8Nc2UyFXcd4kOmOsBIoYtZ2ik"
        }"#;
        let key = Key::from_jwk(jwk).unwrap();
        assert_eq!(
            key.fingerprint(),
            "z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp"
        );
    }

    #[test]
    fn test_from_x25519_jwk() {
        // vector from https://w3c-ccg.github.io/did-method-key/#ed25519-x25519
        let jwk = r#"{
            "kty": "OKP",
            "crv": "X25519",
            "x": "W_Vcc7guviK-gPNDBmevVw-uJVamQV5rMNQGUwCqlH0"
        }"#;
        let key = Key::from_jwk(jwk).unwrap();
        assert_eq!(
            key.fingerprint(),
            "z6LShs9GGnqk85isEBzzshkuVWrVKsRp24GnDuHk8QWkARMW"
        );
    }

    #[test]
    fn test_from_p256_jwk() {
        // vector from https://dev.uniresolver.io/
        let jwk = r#"{
            "kty": "EC",
            "crv": "P-256",
            "x": "fyNYMN0976ci7xqiSdag3buk-ZCwgXU4kz9XNkBlNUI",
            "y": "hW2ojTNfH7Jbi8--CJUo3OCbH3y5n91g-IMA9MLMbTU"
        }"#;
        let key = Key::from_jwk(jwk).unwrap();
        assert_eq!(
            key.fingerprint(),
            "zDnaerDaTF5BXEavCrfRZEk316dpbLsfPDZ3WJ5hRTPFU2169"
        );
    }

    #[test]
    fn test_from_p384_jwk() {
        // vector from https://dev.uniresolver.io/
        let jwk = r#"{
            "kty": "EC",
            "crv": "P-384",
            "x": "bKq-gg3sJmfkJGrLl93bsumOTX1NubBySttAV19y5ClWK3DxEmqPy0at5lLqBiiv",
            "y": "PJQtdHnInU9SY3e8Nn9aOPoP51OFbs-FWJUsU0TGjRtZ4bnhoZXtS92wdzuAotL9"
        }"#;
        let key = Key::from_jwk(jwk).unwrap();
        assert_eq!(
            key.fingerprint(),
            "z82Lkytz3HqpWiBmt2853ZgNgNG8qVoUJnyoMvGw6ZEBktGcwUVdKpUNJHct1wvp9pXjr7Y"
        );
    }
}
