use std::collections::HashMap;

use did_parser::{Did, DidUrl};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::types::{jsonwebkey::JsonWebKey, multibase::Multibase};

// Either a set of verification methods maps or DID URLs
// https://www.w3.org/TR/did-core/#did-document-properties
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum VerificationMethodKind {
    Resolved(VerificationMethod),
    Resolvable(DidUrl),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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
    #[serde(flatten)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    extra: HashMap<String, Value>,
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

    pub fn extra_field(&self, key: &str) -> Option<&Value> {
        self.extra.get(key)
    }
}

#[derive(Debug, Default)]
pub struct IncompleteVerificationMethodBuilder {
    id: DidUrl,
    controller: Did,
    verification_method_type: String,
    extra: HashMap<String, Value>,
}

#[derive(Debug)]
pub struct CompleteVerificationMethodBuilder {
    id: DidUrl,
    controller: Did,
    verification_method_type: String,
    public_key_multibase: Option<Multibase>,
    public_key_jwk: Option<JsonWebKey>,
    extra: HashMap<String, Value>,
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
            public_key_jwk: None,
            extra: self.extra,
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
            public_key_multibase: None,
            public_key_jwk: Some(public_key_jwk),
            extra: self.extra,
        }
    }

    pub fn add_extra_field(mut self, key: String, value: Value) -> Self {
        self.extra.insert(key, value);
        self
    }

    pub fn build(self) -> VerificationMethod {
        VerificationMethod {
            id: self.id,
            controller: self.controller,
            verification_method_type: self.verification_method_type,
            public_key_multibase: None,
            public_key_jwk: None,
            extra: self.extra,
        }
    }
}

impl CompleteVerificationMethodBuilder {
    pub fn add_extra_field(mut self, key: String, value: Value) -> Self {
        self.extra.insert(key, value);
        self
    }

    pub fn build(self) -> VerificationMethod {
        VerificationMethod {
            id: self.id,
            controller: self.controller,
            verification_method_type: self.verification_method_type,
            public_key_multibase: self.public_key_multibase,
            public_key_jwk: self.public_key_jwk,
            extra: self.extra,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_verification_method_extra() {
        let id = create_valid_did_url();
        let controller = create_valid_did();
        let verification_method_type = create_valid_verification_key_type();
        let extra_key = "foo".to_string();
        let extra_value = Value::String("bar".to_string());

        let vm = VerificationMethod::builder(id, controller, verification_method_type)
            .add_extra_field(extra_key.clone(), extra_value.clone())
            .build();
        assert_eq!(vm.extra_field(&extra_key).unwrap(), &extra_value);
    }

    #[test]
    fn test_verification_method_builder_complete() {
        let id = create_valid_did_url();
        let controller = create_valid_did();
        let verification_method_type = create_valid_verification_key_type();
        let public_key_multibase = create_valid_multibase();
        let extra_key = "foo".to_string();
        let extra_value = Value::String("bar".to_string());

        let vm = VerificationMethod::builder(
            id.clone(),
            controller.clone(),
            verification_method_type.clone(),
        )
        .add_public_key_multibase(public_key_multibase.clone())
        .add_extra_field(extra_key.clone(), extra_value.clone())
        .build();

        assert_eq!(vm.id(), &id);
        assert_eq!(vm.controller(), &controller);
        assert_eq!(vm.verification_method_type(), &verification_method_type);
        assert_eq!(vm.public_key_multibase().unwrap(), &public_key_multibase);
        assert_eq!(vm.extra_field(&extra_key).unwrap(), &extra_value);
    }
}
