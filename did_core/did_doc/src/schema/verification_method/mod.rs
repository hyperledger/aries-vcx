pub mod error;
pub mod public_key;
mod verification_method_kind;
mod verification_method_type;

use ::public_key::Key;
use did_parser_nom::{Did, DidUrl};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
pub use verification_method_kind::VerificationMethodKind;
pub use verification_method_type::VerificationMethodType;

pub use self::public_key::PublicKeyField;
use crate::error::DidDocumentBuilderError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct VerificationMethod {
    id: DidUrl,
    controller: Did,
    #[serde(rename = "type")]
    verification_method_type: VerificationMethodType,
    #[serde(flatten)]
    public_key: PublicKeyField,
}

impl VerificationMethod {
    pub fn id(&self) -> &DidUrl {
        &self.id
    }

    pub fn controller(&self) -> &Did {
        &self.controller
    }

    pub fn verification_method_type(&self) -> &VerificationMethodType {
        &self.verification_method_type
    }

    pub fn public_key_field(&self) -> &PublicKeyField {
        &self.public_key
    }

    pub fn public_key(&self) -> Result<Key, DidDocumentBuilderError> {
        let key = match &self.public_key {
            PublicKeyField::Multibase {
                public_key_multibase,
            } => Key::from_fingerprint(public_key_multibase)?,
            #[cfg(feature = "jwk")]
            PublicKeyField::Jwk { public_key_jwk } => Key::from_jwk(&public_key_jwk.to_string())?,
            // TODO - FUTURE - other key types could do with some special handling, i.e.
            // those where the key_type is encoded within the key field (multibase, jwk, etc)
            _ => Key::new(
                self.public_key.key_decoded()?,
                self.verification_method_type.try_into()?,
            )?,
        };

        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    fn create_valid_did() -> Did {
        Did::parse("did:example:123456789abcdefghi".to_string()).unwrap()
    }

    fn create_valid_did_url() -> DidUrl {
        DidUrl::parse("did:example:123456789abcdefghi#fragment".to_string()).unwrap()
    }

    fn create_valid_multibase() -> String {
        "zQmWvQxTqbG2Z9HPJgG57jjwR154cKhbtJenbyYTWkjgF3e".to_string()
    }

    fn create_valid_verification_key_type() -> VerificationMethodType {
        VerificationMethodType::Ed25519VerificationKey2018
    }

    fn create_valid_verification_method_value() -> Value {
        serde_json::json!({
            "id": "did:example:123456789abcdefghi#key-1",
            "type": "Ed25519VerificationKey2018",
            "controller": "did:example:123456789abcdefghi",
            "publicKeyMultibase": "zQmWvQxTqbG2Z9HPJgG57jjwR154cKhbtJenbyYTWkjgF3e"
        })
    }

    fn create_verification_method_multiple_keys() -> Value {
        serde_json::json!({
            "id": "did:example:123456789abcdefghi#key-1",
            "type": "Ed25519VerificationKey2018",
            "controller": "did:example:123456789abcdefghi",
            "publicKeyMultibase": "zQmWvQxTqbG2Z9HPJgG57jjwR154cKhbtJenbyYTWkjgF3e",
            "publicKeyJwk": {
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "zQmWvQxTqbG2Z9HPJgG57jjwR154cKhbtJenbyYTWkjgF3e"
            }
        })
    }

    #[test]
    fn test_verification_method_builder() {
        let id = create_valid_did_url();
        let controller = create_valid_did();
        let verification_method_type = create_valid_verification_key_type();
        let public_key_multibase = create_valid_multibase();

        let vm = VerificationMethod::builder()
            .id(id.clone())
            .controller(controller.clone())
            .verification_method_type(verification_method_type)
            .public_key(PublicKeyField::Multibase {
                public_key_multibase,
            })
            .build();

        assert_eq!(vm.id(), &id);
        assert_eq!(vm.controller(), &controller);
        assert_eq!(vm.verification_method_type(), &verification_method_type);
        match vm.public_key_field() {
            PublicKeyField::Multibase {
                public_key_multibase,
            } => {
                assert_eq!(public_key_multibase, public_key_multibase)
            }
            _ => panic!("Expected public key to be multibase"),
        }
    }

    #[test]
    fn test_verification_method_deserialization() {
        let vm: Result<VerificationMethod, _> = serde_json::from_str(
            create_valid_verification_method_value()
                .to_string()
                .as_str(),
        );
        assert!(vm.is_ok());
    }

    #[test]
    fn test_verification_method_deserialization_fails_with_multiple_keys() {
        let vm: Result<VerificationMethod, _> = serde_json::from_str(
            create_verification_method_multiple_keys()
                .to_string()
                .as_str(),
        );
        assert!(vm.is_err());
    }
}

#[cfg(feature = "jwk")]
#[cfg(test)]
mod jwk_tests {
    use ::public_key::KeyType;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_public_key_from_ed25519_jwk_vm() {
        let vm: VerificationMethod = serde_json::from_value(json!({
            "id": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH#z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
            "type": "Ed25519VerificationKey2018",
            "controller": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
            "publicKeyJwk": {
              "kty": "OKP",
              "crv": "Ed25519",
              "x": "lJZrfAjkBXdfjebMHEUI9usidAPhAlssitLXR3OYxbI"
            }
          })).unwrap();
        let pk = vm.public_key().unwrap();
        assert!(matches!(pk.key_type(), KeyType::Ed25519));
        assert_eq!(
            pk.fingerprint(),
            "z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH"
        )
    }

    #[test]
    fn test_public_key_from_p256_jwk_vm() {
        let vm: VerificationMethod = serde_json::from_value(json!({
            "id": "did:key:zDnaerDaTF5BXEavCrfRZEk316dpbLsfPDZ3WJ5hRTPFU2169#zDnaerDaTF5BXEavCrfRZEk316dpbLsfPDZ3WJ5hRTPFU2169",
            "type": "JsonWebKey2020",
            "controller": "did:key:zDnaerDaTF5BXEavCrfRZEk316dpbLsfPDZ3WJ5hRTPFU2169",
            "publicKeyJwk": {
              "kty": "EC",
              "crv": "P-256",
              "x": "fyNYMN0976ci7xqiSdag3buk-ZCwgXU4kz9XNkBlNUI",
              "y": "hW2ojTNfH7Jbi8--CJUo3OCbH3y5n91g-IMA9MLMbTU"
            }
          })).unwrap();
        let pk = vm.public_key().unwrap();
        assert!(matches!(pk.key_type(), KeyType::P256));
        assert_eq!(
            pk.fingerprint(),
            "zDnaerDaTF5BXEavCrfRZEk316dpbLsfPDZ3WJ5hRTPFU2169"
        )
    }
}
