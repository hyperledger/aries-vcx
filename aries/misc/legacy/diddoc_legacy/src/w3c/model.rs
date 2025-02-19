use serde::{Serialize, Serializer};

pub const CONTEXT: &str = "https://w3id.org/did/v1";
pub const KEY_TYPE: &str = "Ed25519VerificationKey2018"; // TODO: Should be Ed25519Signature2018?
pub const KEY_AUTHENTICATION_TYPE: &str = "Ed25519SignatureAuthentication2018";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Ed25519PublicKey {
    pub id: String,
    // all list of types: https://w3c-ccg.github.io/ld-cryptosuite-registry/
    #[serde(rename = "type")]
    pub type_: String,
    pub controller: String,
    #[serde(rename = "publicKeyBase58")]
    pub public_key_base_58: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Authentication {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

#[derive(Debug, PartialEq)]
pub struct DdoKeyReference {
    pub did: Option<String>,
    pub key_id: String,
}

impl Serialize for DdoKeyReference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.did {
            None => serializer.collect_str(&self.key_id),
            Some(did) => serializer.collect_str(&format!("{}#{}", did, self.key_id)),
        }
    }
}

#[cfg(test)]
mod unit_test {
    use crate::{aries::diddoc::test_utils::_did, w3c::model::DdoKeyReference};

    #[test]
    fn test_key_reference_serialization() {
        let key_ref = DdoKeyReference {
            did: Some(_did()),
            key_id: "1".to_string(),
        };
        let serialized = serde_json::to_string(&key_ref).unwrap();
        assert_eq!(format!("\"{}#1\"", _did()), serialized)
    }
}
