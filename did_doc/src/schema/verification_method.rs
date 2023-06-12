use did_parser::{Did, DidUrl};
use serde::{de, Deserialize, Deserializer, Serialize};

use super::types::{jsonwebkey::JsonWebKey, multibase::Multibase};

// Either a set of verification methods maps or DID URLs
// https://www.w3.org/TR/did-core/#did-document-properties
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum VerificationMethodKind {
    Resolved(VerificationMethod),
    Resolvable(DidUrl),
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VerificationMethod {
    id: DidUrl,
    controller: Did,
    #[serde(rename = "type")]
    verification_method_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    public_key_multibase: Option<Multibase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    public_key_jwk: Option<JsonWebKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    public_key_base58: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    public_key_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    public_key_hex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    public_key_pem: Option<String>,
}

impl VerificationMethod {
    pub fn builder(
        id: DidUrl,
        controller: Did,
        verification_method_type: String,
    ) -> IncompleteVerificationMethodBuilder {
        IncompleteVerificationMethodBuilder::new(id, controller, verification_method_type)
    }

    pub fn id(&self) -> &DidUrl {
        &self.id
    }

    pub fn controller(&self) -> &Did {
        &self.controller
    }

    pub fn verification_method_type(&self) -> &str {
        self.verification_method_type.as_ref()
    }

    pub fn public_key_multibase(&self) -> Option<&Multibase> {
        self.public_key_multibase.as_ref()
    }

    pub fn public_key_jwk(&self) -> Option<&JsonWebKey> {
        self.public_key_jwk.as_ref()
    }

    pub fn public_key_base58(&self) -> Option<&String> {
        self.public_key_base58.as_ref()
    }

    pub fn public_key_base64(&self) -> Option<&String> {
        self.public_key_base64.as_ref()
    }

    pub fn public_key_hex(&self) -> Option<&String> {
        self.public_key_hex.as_ref()
    }

    pub fn public_key_pem(&self) -> Option<&String> {
        self.public_key_pem.as_ref()
    }
}

#[derive(Debug, Default)]
pub struct IncompleteVerificationMethodBuilder {
    id: DidUrl,
    controller: Did,
    verification_method_type: String,
}

#[derive(Debug, Default)]
pub struct CompleteVerificationMethodBuilder {
    id: DidUrl,
    controller: Did,
    verification_method_type: String,
    public_key_multibase: Option<Multibase>,
    public_key_jwk: Option<JsonWebKey>,
    public_key_base58: Option<String>,
    public_key_base64: Option<String>,
    public_key_hex: Option<String>,
    public_key_pem: Option<String>,
}

impl IncompleteVerificationMethodBuilder {
    pub fn new(id: DidUrl, controller: Did, verification_method_type: String) -> Self {
        Self {
            id,
            verification_method_type,
            controller,
            ..Default::default()
        }
    }

    pub fn add_public_key_multibase(
        self,
        public_key_multibase: Multibase,
    ) -> CompleteVerificationMethodBuilder {
        CompleteVerificationMethodBuilder {
            id: self.id,
            controller: self.controller,
            verification_method_type: self.verification_method_type,
            public_key_multibase: Some(public_key_multibase),
            ..Default::default()
        }
    }

    pub fn add_public_key_jwk(
        self,
        public_key_jwk: JsonWebKey,
    ) -> CompleteVerificationMethodBuilder {
        CompleteVerificationMethodBuilder {
            id: self.id,
            controller: self.controller,
            verification_method_type: self.verification_method_type,
            public_key_jwk: Some(public_key_jwk),
            ..Default::default()
        }
    }

    pub fn add_public_key_base58(
        self,
        public_key_base58: String,
    ) -> CompleteVerificationMethodBuilder {
        CompleteVerificationMethodBuilder {
            id: self.id,
            controller: self.controller,
            verification_method_type: self.verification_method_type,
            public_key_base58: Some(public_key_base58),
            ..Default::default()
        }
    }

    pub fn add_public_key_base64(
        self,
        public_key_base64: String,
    ) -> CompleteVerificationMethodBuilder {
        CompleteVerificationMethodBuilder {
            id: self.id,
            controller: self.controller,
            verification_method_type: self.verification_method_type,
            public_key_base64: Some(public_key_base64),
            ..Default::default()
        }
    }

    pub fn add_public_key_hex(self, public_key_hex: String) -> CompleteVerificationMethodBuilder {
        CompleteVerificationMethodBuilder {
            id: self.id,
            controller: self.controller,
            verification_method_type: self.verification_method_type,
            public_key_hex: Some(public_key_hex),
            ..Default::default()
        }
    }

    pub fn add_public_key_pem(self, public_key_pem: String) -> CompleteVerificationMethodBuilder {
        CompleteVerificationMethodBuilder {
            id: self.id,
            controller: self.controller,
            verification_method_type: self.verification_method_type,
            public_key_pem: Some(public_key_pem),
            ..Default::default()
        }
    }
}

impl CompleteVerificationMethodBuilder {
    pub fn build(self) -> VerificationMethod {
        VerificationMethod {
            id: self.id,
            controller: self.controller,
            verification_method_type: self.verification_method_type,
            public_key_multibase: self.public_key_multibase,
            public_key_jwk: self.public_key_jwk,
            public_key_base58: self.public_key_base58,
            public_key_base64: self.public_key_base64,
            public_key_hex: self.public_key_hex,
            public_key_pem: self.public_key_pem,
        }
    }
}

impl<'de> Deserialize<'de> for VerificationMethod {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        struct VerificationMethodTmp {
            id: DidUrl,
            controller: Did,
            #[serde(rename = "type")]
            verification_method_type: String,
            #[serde(default)]
            public_key_multibase: Option<Multibase>,
            #[serde(default)]
            public_key_jwk: Option<JsonWebKey>,
            #[serde(default)]
            public_key_base58: Option<String>,
            #[serde(default)]
            public_key_base64: Option<String>,
            #[serde(default)]
            public_key_hex: Option<String>,
            #[serde(default)]
            public_key_pem: Option<String>,
        }

        let tmp = VerificationMethodTmp::deserialize(deserializer)?;

        let mut count = 0;
        if tmp.public_key_multibase.is_some() {
            count += 1;
        }
        if tmp.public_key_jwk.is_some() {
            count += 1;
        }
        if tmp.public_key_base58.is_some() {
            count += 1;
        }
        if tmp.public_key_base64.is_some() {
            count += 1;
        }
        if tmp.public_key_hex.is_some() {
            count += 1;
        }
        if tmp.public_key_pem.is_some() {
            count += 1;
        }

        if count != 1 {
            Err(de::Error::custom(format!(
                "Expected exactly one `public_key` field, but found {}",
                count
            )))
        } else {
            Ok(VerificationMethod {
                id: tmp.id,
                controller: tmp.controller,
                verification_method_type: tmp.verification_method_type,
                public_key_multibase: tmp.public_key_multibase,
                public_key_jwk: tmp.public_key_jwk,
                public_key_base58: tmp.public_key_base58,
                public_key_base64: tmp.public_key_base64,
                public_key_hex: tmp.public_key_hex,
                public_key_pem: tmp.public_key_pem,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn create_valid_did() -> Did {
        Did::parse("did:example:123456789abcdefghi".to_string()).unwrap()
    }

    fn create_valid_did_url() -> DidUrl {
        DidUrl::parse("did:example:123456789abcdefghi#fragment".to_string()).unwrap()
    }

    fn create_valid_multibase() -> Multibase {
        Multibase::new("zQmWvQxTqbG2Z9HPJgG57jjwR154cKhbtJenbyYTWkjgF3e".to_string()).unwrap()
    }

    fn create_valid_verification_key_type() -> String {
        "Ed25519VerificationKey2018".to_string()
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
    fn test_verification_method_id() {
        let id = create_valid_did_url();
        let controller = create_valid_did();
        let verification_method_type = create_valid_verification_key_type();
        let verification_method =
            VerificationMethod::builder(id.clone(), controller, verification_method_type)
                .add_public_key_multibase(create_valid_multibase())
                .build();
        assert_eq!(verification_method.id(), &id);
    }

    #[test]
    fn test_verification_method_builder() {
        let id = create_valid_did_url();
        let controller = create_valid_did();
        let verification_method_type = create_valid_verification_key_type();
        let public_key_multibase = create_valid_multibase();

        let vm = VerificationMethod::builder(
            id.clone(),
            controller.clone(),
            verification_method_type.clone(),
        )
        .add_public_key_multibase(public_key_multibase.clone())
        .build();

        assert_eq!(vm.id(), &id);
        assert_eq!(vm.controller(), &controller);
        assert_eq!(vm.verification_method_type(), &verification_method_type);
        assert_eq!(vm.public_key_multibase().unwrap(), &public_key_multibase);
    }

    #[test]
    fn test_verification_method_builder_complete() {
        let id = create_valid_did_url();
        let controller = create_valid_did();
        let verification_method_type = create_valid_verification_key_type();
        let public_key_multibase = create_valid_multibase();

        let vm = VerificationMethod::builder(
            id.clone(),
            controller.clone(),
            verification_method_type.clone(),
        )
        .add_public_key_multibase(public_key_multibase.clone())
        .build();

        assert_eq!(vm.id(), &id);
        assert_eq!(vm.controller(), &controller);
        assert_eq!(vm.verification_method_type(), &verification_method_type);
        assert_eq!(vm.public_key_multibase().unwrap(), &public_key_multibase);
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
