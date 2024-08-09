use base64::{engine::general_purpose, Engine};
use public_key::Key;
use serde::{Deserialize, Serialize};

use crate::schema::{types::jsonwebkey::JsonWebKey, verification_method::error::KeyDecodingError};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
pub enum PublicKeyField {
    #[serde(rename_all = "camelCase")]
    Multibase { public_key_multibase: String },
    #[serde(rename_all = "camelCase")]
    Jwk { public_key_jwk: JsonWebKey },
    #[serde(rename_all = "camelCase")]
    Base58 { public_key_base58: String },
    #[serde(rename_all = "camelCase")]
    Base64 { public_key_base64: String },
    #[serde(rename_all = "camelCase")]
    Hex { public_key_hex: String },
    #[serde(rename_all = "camelCase")]
    Pem { public_key_pem: String },
    #[serde(rename_all = "camelCase")]
    Pgp { public_key_pgp: String },
}

impl PublicKeyField {
    pub fn key_decoded(&self) -> Result<Vec<u8>, KeyDecodingError> {
        match self {
            PublicKeyField::Multibase {
                public_key_multibase,
            } => {
                let key = Key::from_fingerprint(public_key_multibase)?;
                Ok(key.key().to_vec())
            }
            PublicKeyField::Base58 { public_key_base58 } => {
                Ok(bs58::decode(public_key_base58).into_vec()?)
            }
            PublicKeyField::Base64 { public_key_base64 } => {
                Ok(general_purpose::STANDARD_NO_PAD.decode(public_key_base64.as_bytes())?)
            }
            PublicKeyField::Hex { public_key_hex } => Ok(hex::decode(public_key_hex)?),
            PublicKeyField::Pem { public_key_pem } => {
                Ok(pem::parse(public_key_pem.as_bytes())?.contents().to_vec())
            }
            PublicKeyField::Jwk { public_key_jwk: _ } => Err(KeyDecodingError::new(
                "JWK public key decoding not supported",
            )),
            PublicKeyField::Pgp { public_key_pgp: _ } => Err(KeyDecodingError::new(
                "PGP public key decoding not supported",
            )),
        }
    }

    // TODO: Other formats
    pub fn base58(&self) -> Result<String, KeyDecodingError> {
        Ok(bs58::encode(self.key_decoded()?).into_string())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    static PUBLIC_KEY_MULTIBASE: &str = "z6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc";
    static PUBLIC_KEY_BASE58: &str = "JhNWeSVLMYccCk7iopQW4guaSJTojqpMEELgSLhKwRr";
    static PUBLIC_KEY_BASE64: &str = "BIiFcQEn3dfvB2pjlhOQQour6jXy9d5s2FKEJNTOJik";
    static PUBLIC_KEY_HEX: &str =
        "048885710127ddd7ef076a63961390428babea35f2f5de6cd8528424d4ce2629";
    static PUBLIC_KEY_BYTES: [u8; 32] = [
        4, 136, 133, 113, 1, 39, 221, 215, 239, 7, 106, 99, 150, 19, 144, 66, 139, 171, 234, 53,
        242, 245, 222, 108, 216, 82, 132, 36, 212, 206, 38, 41,
    ];

    #[test]
    fn test_multibase() {
        let public_key_field = PublicKeyField::Multibase {
            public_key_multibase: PUBLIC_KEY_MULTIBASE.to_string(),
        };
        assert_eq!(public_key_field.key_decoded().unwrap(), PUBLIC_KEY_BYTES);
        assert_eq!(public_key_field.base58().unwrap(), PUBLIC_KEY_BASE58);
    }

    #[test]
    fn test_base58() {
        let public_key_field = PublicKeyField::Base58 {
            public_key_base58: PUBLIC_KEY_BASE58.to_string(),
        };
        assert_eq!(
            public_key_field.key_decoded().unwrap(),
            PUBLIC_KEY_BYTES.to_vec()
        );
    }

    #[test]
    fn test_base64() {
        let public_key_field = PublicKeyField::Base64 {
            public_key_base64: PUBLIC_KEY_BASE64.to_string(),
        };
        assert_eq!(
            public_key_field.key_decoded().unwrap(),
            PUBLIC_KEY_BYTES.to_vec()
        );
    }

    #[test]
    fn test_hex() {
        let public_key_field = PublicKeyField::Hex {
            public_key_hex: PUBLIC_KEY_HEX.to_string(),
        };
        assert_eq!(
            public_key_field.key_decoded().unwrap(),
            PUBLIC_KEY_BYTES.to_vec()
        );
    }

    #[test]
    fn test_b58_fails() {
        let public_key_field = PublicKeyField::Base58 {
            public_key_base58: "abcdefghijkl".to_string(),
        };
        let err = public_key_field.key_decoded().expect_err("Expected error");
        println!("Error: {}", err);
        assert!(err
            .source()
            .expect("Error was expected to has source set up.")
            .is::<bs58::decode::Error>());
        assert!(err.to_string().contains("Failed to decode base58"));
    }

    #[test]
    fn test_pem_fails() {
        let public_key_field = PublicKeyField::Pem {
            public_key_pem: "abcdefghijkl".to_string(),
        };
        let err = public_key_field.key_decoded().unwrap_err();
        println!("Error: {}", err);
        assert!(err
            .source()
            .expect("Error was expected to has source set up.")
            .is::<pem::PemError>());
        assert!(err.to_string().contains("Failed to decode PEM"));
    }
}
