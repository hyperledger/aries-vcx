use std::collections::HashMap;

use did_parser_nom::{Did, DidUrl};
use display_as_json::Display;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    types::uri::Uri,
    utils::OneOrList,
    verification_method::{VerificationMethod, VerificationMethodKind},
};
use crate::{error::DidDocumentBuilderError, schema::service::Service};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, Display)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct DidDocument {
    id: Did,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    also_known_as: Vec<Uri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    controller: Option<OneOrList<Did>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    verification_method: Vec<VerificationMethod>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    authentication: Vec<VerificationMethodKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    assertion_method: Vec<VerificationMethodKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    key_agreement: Vec<VerificationMethodKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    capability_invocation: Vec<VerificationMethodKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    capability_delegation: Vec<VerificationMethodKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    service: Vec<Service>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

impl DidDocument {
    pub fn builder(id: Did) -> DidDocumentBuilder {
        DidDocumentBuilder::new(id)
    }

    pub fn id(&self) -> &Did {
        &self.id
    }

    pub fn set_id(&mut self, id: Did) {
        self.id = id;
    }

    pub fn also_known_as(&self) -> &[Uri] {
        self.also_known_as.as_ref()
    }

    pub fn controller(&self) -> Option<&OneOrList<Did>> {
        self.controller.as_ref()
    }

    pub fn verification_method(&self) -> &[VerificationMethod] {
        self.verification_method.as_ref()
    }

    pub fn authentication(&self) -> &[VerificationMethodKind] {
        self.authentication.as_ref()
    }

    pub fn assertion_method(&self) -> &[VerificationMethodKind] {
        self.assertion_method.as_ref()
    }

    pub fn key_agreement(&self) -> &[VerificationMethodKind] {
        self.key_agreement.as_ref()
    }

    pub fn capability_invocation(&self) -> &[VerificationMethodKind] {
        self.capability_invocation.as_ref()
    }

    pub fn capability_delegation(&self) -> &[VerificationMethodKind] {
        self.capability_delegation.as_ref()
    }

    pub fn service(&self) -> &[Service] {
        self.service.as_ref()
    }

    pub fn extra_field(&self, key: &str) -> Option<&Value> {
        self.extra.get(key)
    }

    pub fn dereference_key(&self, reference: &DidUrl) -> Option<&VerificationMethod> {
        self.verification_method
            .iter()
            .find(|vm| vm.id().fragment() == reference.fragment())
    }

    pub fn validate(&self) -> Result<(), DidDocumentBuilderError> {
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct DidDocumentBuilder {
    id: Did,
    also_known_as: Vec<Uri>,
    controller: Vec<Did>,
    verification_method: Vec<VerificationMethod>,
    authentication: Vec<VerificationMethodKind>,
    assertion_method: Vec<VerificationMethodKind>,
    key_agreement: Vec<VerificationMethodKind>,
    capability_invocation: Vec<VerificationMethodKind>,
    capability_delegation: Vec<VerificationMethodKind>,
    service: Vec<Service>,
    extra: HashMap<String, Value>,
}

impl DidDocumentBuilder {
    pub fn new(id: Did) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn add_also_known_as(mut self, also_known_as: Uri) -> Self {
        self.also_known_as.push(also_known_as);
        self
    }

    pub fn add_controller(mut self, controller: Did) -> Self {
        self.controller.push(controller);
        self
    }

    pub fn add_verification_method(mut self, verification_method: VerificationMethod) -> Self {
        self.verification_method.push(verification_method);
        self
    }

    pub fn add_authentication_method(mut self, method: VerificationMethod) -> Self {
        self.authentication
            .push(VerificationMethodKind::Resolved(method));
        self
    }

    pub fn add_authentication_reference(mut self, reference: DidUrl) -> Self {
        self.authentication
            .push(VerificationMethodKind::Resolvable(reference));
        self
    }

    pub fn add_assertion_method(mut self, method: VerificationMethod) -> Self {
        self.assertion_method
            .push(VerificationMethodKind::Resolved(method));
        self
    }

    pub fn add_assertion_method_reference(mut self, reference: DidUrl) -> Self {
        self.assertion_method
            .push(VerificationMethodKind::Resolvable(reference));
        self
    }

    pub fn add_key_agreement(mut self, key_agreement: VerificationMethod) -> Self {
        self.key_agreement
            .push(VerificationMethodKind::Resolved(key_agreement));
        self
    }

    pub fn add_key_agreement_reference(mut self, reference: DidUrl) -> Self {
        self.key_agreement
            .push(VerificationMethodKind::Resolvable(reference));
        self
    }

    pub fn add_capability_invocation(mut self, capability_invocation: VerificationMethod) -> Self {
        self.capability_invocation
            .push(VerificationMethodKind::Resolved(capability_invocation));
        self
    }

    pub fn add_capability_invocation_reference(mut self, reference: DidUrl) -> Self {
        self.capability_invocation
            .push(VerificationMethodKind::Resolvable(reference));
        self
    }

    pub fn add_capability_delegation(mut self, capability_delegation: VerificationMethod) -> Self {
        self.capability_delegation
            .push(VerificationMethodKind::Resolved(capability_delegation));
        self
    }

    pub fn add_capability_delegation_refrence(mut self, reference: DidUrl) -> Self {
        self.capability_delegation
            .push(VerificationMethodKind::Resolvable(reference));
        self
    }

    pub fn add_service(mut self, service: Service) -> Self {
        self.service.push(service);
        self
    }

    pub fn add_extra_field(mut self, key: String, value: Value) -> Self {
        self.extra.insert(key, value);
        self
    }

    pub fn build(self) -> DidDocument {
        let controller = if self.controller.is_empty() {
            None
        } else {
            Some(OneOrList::List(self.controller))
        };
        DidDocument {
            id: self.id,
            also_known_as: self.also_known_as,
            controller,
            verification_method: self.verification_method,
            authentication: self.authentication,
            assertion_method: self.assertion_method,
            key_agreement: self.key_agreement,
            capability_invocation: self.capability_invocation,
            capability_delegation: self.capability_delegation,
            service: self.service,
            extra: self.extra,
        }
    }
}

impl From<DidDocument> for DidDocumentBuilder {
    fn from(did_document: DidDocument) -> Self {
        let controller = match did_document.controller {
            Some(OneOrList::List(list)) => list,
            _ => Vec::new(),
        };

        Self {
            id: did_document.id,
            also_known_as: did_document.also_known_as,
            controller,
            verification_method: did_document.verification_method,
            authentication: did_document.authentication,
            assertion_method: did_document.assertion_method,
            key_agreement: did_document.key_agreement,
            capability_invocation: did_document.capability_invocation,
            capability_delegation: did_document.capability_delegation,
            service: did_document.service,
            extra: did_document.extra,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::verification_method::{PublicKeyField, VerificationMethodType};

    #[test]
    fn test_did_document_builder() {
        let id = Did::parse("did:example:123456789abcdefghi".to_string()).unwrap();
        let also_known_as = Uri::new("https://example.com").unwrap();
        let controller = Did::parse("did:example:controller".to_string()).unwrap();

        let vm1_id = DidUrl::parse("did:example:vm1#vm1".to_string()).unwrap();
        let verification_method = VerificationMethod::builder(
            vm1_id.clone(),
            Did::parse("did:example:vm1".to_string()).unwrap(),
            VerificationMethodType::Ed25519VerificationKey2018,
        )
        .add_public_key_base58("H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string())
        .build();
        let authentication_reference = DidUrl::parse("did:example:authref".to_string()).unwrap();
        let assertion_method = VerificationMethod::builder(
            DidUrl::parse("did:example:am1".to_string()).unwrap(),
            Did::parse("did:example:am2".to_string()).unwrap(),
            VerificationMethodType::Ed25519VerificationKey2018,
        )
        .add_public_key_base58("H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string())
        .build();

        let service_id = Uri::new("did:example:123456789abcdefghi;service-1").unwrap();
        let service_endpoint = "https://example.com/service";
        let service = Service::new(
            service_id,
            service_endpoint.try_into().unwrap(),
            OneOrList::One(ServiceType::Other("test-service".to_string())),
            HashMap::default(),
        );

        let document = DidDocumentBuilder::new(id.clone())
            .add_also_known_as(also_known_as.clone())
            .add_controller(controller.clone())
            .add_verification_method(verification_method.clone())
            .add_authentication_method(verification_method.clone())
            .add_authentication_reference(authentication_reference.clone())
            .add_assertion_method(assertion_method.clone())
            .add_assertion_method_reference(authentication_reference.clone())
            .add_key_agreement(verification_method.clone())
            .add_key_agreement_reference(authentication_reference.clone())
            .add_capability_invocation(verification_method.clone())
            .add_capability_invocation_reference(authentication_reference.clone())
            .add_capability_delegation(verification_method.clone())
            .add_capability_delegation_refrence(authentication_reference.clone())
            .add_service(service.clone())
            .build();

        assert_eq!(document.id(), &id);
        assert_eq!(document.also_known_as(), &[also_known_as]);
        assert_eq!(
            document.controller(),
            Some(&OneOrList::List(vec![controller]))
        );
        assert_eq!(
            document.verification_method(),
            &[verification_method.clone()]
        );
        assert_eq!(
            document.authentication(),
            &[
                VerificationMethodKind::Resolved(verification_method.clone()),
                VerificationMethodKind::Resolvable(authentication_reference.clone())
            ]
        );
        assert_eq!(
            document.assertion_method(),
            &[
                VerificationMethodKind::Resolved(assertion_method),
                VerificationMethodKind::Resolvable(authentication_reference.clone())
            ]
        );
        assert_eq!(
            document.key_agreement(),
            &[
                VerificationMethodKind::Resolved(verification_method.clone()),
                VerificationMethodKind::Resolvable(authentication_reference.clone())
            ]
        );
        assert_eq!(
            document.capability_invocation(),
            &[
                VerificationMethodKind::Resolved(verification_method.clone()),
                VerificationMethodKind::Resolvable(authentication_reference.clone())
            ]
        );
        assert_eq!(
            document.capability_delegation(),
            &[
                VerificationMethodKind::Resolved(verification_method),
                VerificationMethodKind::Resolvable(authentication_reference)
            ]
        );
        assert_eq!(document.service(), &[service]);

        let vm = document.dereference_key(&vm1_id);
        if let Some(vm) = vm {
            assert_eq!(vm.id(), &vm1_id);
        } else {
            panic!("Verification method not found")
        };
    }

    use std::str::FromStr;

    use did_parser_nom::{Did, DidUrl};
    use serde_json::Value;

    use crate::schema::{
        did_doc::DidDocument,
        service::typed::ServiceType,
        types::{jsonwebkey::JsonWebKey, uri::Uri},
        verification_method::{VerificationMethod, VerificationMethodKind},
    };

    const VALID_DID_DOC_JSON: &str = r##"
    {
      "@context": [
        "https://w3.org/ns/did/v1",
        "https://w3id.org/security/suites/ed25519-2018/v1"
      ],
      "id": "did:web:did-actor-alice",
      "alsoKnownAs": [
          "https://example.com/user-profile/123"
      ],
      "publicKey": [
        {
          "id": "did:web:did-actor-alice#z6MkrmNwty5ajKtFqc1U48oL2MMLjWjartwc5sf2AihZwXDN",
          "controller": "did:web:did-actor-alice",
          "type": "Ed25519VerificationKey2018",
          "publicKeyBase58": "DK7uJiq9PnPnj7AmNZqVBFoLuwTjT1hFPrk6LSjZ2JRz"
        }
      ],
      "authentication": [
        "did:web:did-actor-alice#z6MkrmNwty5ajKtFqc1U48oL2MMLjWjartwc5sf2AihZwXDN"
      ],
      "assertionMethod": [
        "did:web:did-actor-alice#z6MkrmNwty5ajKtFqc1U48oL2MMLjWjartwc5sf2AihZwXDN"
      ],
      "capabilityDelegation": [
        "did:web:did-actor-alice#z6MkrmNwty5ajKtFqc1U48oL2MMLjWjartwc5sf2AihZwXDN"
      ],
      "capabilityInvocation": [
        "did:web:did-actor-alice#z6MkrmNwty5ajKtFqc1U48oL2MMLjWjartwc5sf2AihZwXDN"
      ],
      "verificationMethod": [
        {
          "id": "#g1",
          "controller": "did:web:did-actor-alice",
          "type": "JsonWebKey2020",
          "publicKeyJwk": {
            "kty": "EC",
            "crv": "BLS12381_G1",
            "x": "hxF12gtsn9ju4-kJq2-nUjZQKVVWpcBAYX5VHnUZMDilClZsGuOaDjlXS8pFE1GG"
          }
        },
        {
          "id": "#g2",
          "controller": "did:web:did-actor-alice",
          "type": "JsonWebKey2020",
          "publicKeyJwk": {
            "kty": "EC",
            "crv": "BLS12381_G2",
            "x": "l4MeBsn_OGa2OEDtHeHdq0TBC8sYh6QwoI7QsNtZk9oAru1OnGClaAPlMbvvs73EABDB6GjjzybbOHarkBmP6pon8H1VuMna0nkEYihZi8OodgdbwReDiDvWzZuXXMl-"
          }
        }
      ],
      "keyAgreement": [
        {
          "id": "did:web:did-actor-alice#zC8GybikEfyNaausDA4mkT4egP7SNLx2T1d1kujLQbcP6h",
          "type": "X25519KeyAgreementKey2019",
          "controller": "did:web:did-actor-alice",
          "publicKeyBase58": "CaSHXEvLKS6SfN9aBfkVGBpp15jSnaHazqHgLHp8KZ3Y"
        }
      ]
    }
    "##;

    #[test]
    fn test_deserialization() {
        let did_doc: DidDocument = serde_json::from_str(VALID_DID_DOC_JSON).unwrap();

        assert_eq!(
            did_doc.id(),
            &"did:web:did-actor-alice".to_string().try_into().unwrap()
        );
        assert_eq!(
            did_doc.also_known_as(),
            vec![Uri::from_str("https://example.com/user-profile/123").unwrap()]
        );

        let controller: Did = "did:web:did-actor-alice".to_string().try_into().unwrap();

        let pk_id = DidUrl::parse(
            "did:web:did-actor-alice#z6MkrmNwty5ajKtFqc1U48oL2MMLjWjartwc5sf2AihZwXDN".to_string(),
        )
        .unwrap();

        let vm1_id = DidUrl::parse("#g1".to_string()).unwrap();
        let vm1 = VerificationMethod::builder(
            vm1_id,
            controller.clone(),
            VerificationMethodType::JsonWebKey2020,
        )
        .add_public_key_jwk(
            JsonWebKey::from_str(
                r#"{
                "kty": "EC",
                "crv": "BLS12381_G1",
                "x": "hxF12gtsn9ju4-kJq2-nUjZQKVVWpcBAYX5VHnUZMDilClZsGuOaDjlXS8pFE1GG"
            }"#,
            )
            .unwrap(),
        )
        .build();

        let vm2_id = DidUrl::parse("#g2".to_string()).unwrap();
        let vm2 = VerificationMethod::builder(
            vm2_id,
            controller.clone(),
            VerificationMethodType::JsonWebKey2020,
        )
            .add_public_key_jwk(
                JsonWebKey::from_str(
                    r#"{
                "kty": "EC",
                "crv": "BLS12381_G2",
                "x": "l4MeBsn_OGa2OEDtHeHdq0TBC8sYh6QwoI7QsNtZk9oAru1OnGClaAPlMbvvs73EABDB6GjjzybbOHarkBmP6pon8H1VuMna0nkEYihZi8OodgdbwReDiDvWzZuXXMl-"
            }"#,
                )
                    .unwrap(),
            )
            .build();

        assert_eq!(did_doc.verification_method().first().unwrap().clone(), vm1);
        assert_eq!(did_doc.verification_method().get(1).unwrap().clone(), vm2);

        assert_eq!(
            did_doc.authentication(),
            &[VerificationMethodKind::Resolvable(pk_id.clone())]
        );

        assert_eq!(
            did_doc.assertion_method(),
            &[VerificationMethodKind::Resolvable(pk_id.clone())]
        );

        assert_eq!(
            did_doc.capability_delegation(),
            &[VerificationMethodKind::Resolvable(pk_id.clone())]
        );

        assert_eq!(
            did_doc.capability_invocation(),
            &[VerificationMethodKind::Resolvable(pk_id)]
        );

        assert_eq!(
            did_doc.extra_field("publicKey").unwrap().clone(),
            Value::Array(vec![Value::Object(
                serde_json::from_str(
                    r#"{
                    "id": "did:web:did-actor-alice#z6MkrmNwty5ajKtFqc1U48oL2MMLjWjartwc5sf2AihZwXDN",
                    "type": "Ed25519VerificationKey2018",
                    "controller": "did:web:did-actor-alice",
                    "publicKeyBase58": "DK7uJiq9PnPnj7AmNZqVBFoLuwTjT1hFPrk6LSjZ2JRz"
                }"#
                )
                    .unwrap()
            )])
        );

        let ka1_id = DidUrl::parse(
            "did:web:did-actor-alice#zC8GybikEfyNaausDA4mkT4egP7SNLx2T1d1kujLQbcP6h".to_string(),
        )
        .unwrap();
        let ka1 = VerificationMethod::builder(
            ka1_id,
            controller,
            VerificationMethodType::X25519KeyAgreementKey2019,
        )
        .add_public_key_base58("CaSHXEvLKS6SfN9aBfkVGBpp15jSnaHazqHgLHp8KZ3Y".to_string())
        .build();

        assert_eq!(
            did_doc.key_agreement(),
            &[VerificationMethodKind::Resolved(ka1)]
        );
    }

    #[test]
    fn test_serialization() {
        let did_doc: DidDocument = serde_json::from_str(VALID_DID_DOC_JSON).unwrap();

        let serialized_json = serde_json::to_string(&did_doc).unwrap();

        let original_json_value: DidDocument = serde_json::from_str(VALID_DID_DOC_JSON).unwrap();
        let serialized_json_value: DidDocument = serde_json::from_str(&serialized_json).unwrap();
        assert_eq!(serialized_json_value, original_json_value);
    }

    #[test]
    fn did_doc_dereferencing() {
        let did_doc: DidDocument = serde_json::from_value(serde_json::json!({
          "@context": [
            "https://w3.org/ns/did/v1",
            "https://w3id.org/security/suites/ed25519-2018/v1"
          ],
          "id": "did:web:did-actor-alice",
          "alsoKnownAs": [
              "https://example.com/user-profile/123"
          ],
          "verificationMethod": [
              {
                  "id": "did:example:123456789abcdefghi#keys-2",
                  "type": "Ed25519VerificationKey2020",
                  "controller": "did:example:123456789abcdefghi",
                  "publicKeyMultibase": "zH3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV"
              },
              {
                  "id": "#keys-3",
                  "type": "Ed25519VerificationKey2020",
                  "controller": "did:example:123456789abcdefghi",
                  "publicKeyMultibase": "zH3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV"
              }
            ]
        }))
        .unwrap();
        {
            let vm = did_doc
                .dereference_key(
                    &DidUrl::parse("did:example:123456789abcdefghi#keys-2".to_string()).unwrap(),
                )
                .unwrap();

            assert_eq!(vm.id().to_string(), "did:example:123456789abcdefghi#keys-2");
            assert_eq!(
                vm.controller().to_string(),
                "did:example:123456789abcdefghi"
            );
            assert_eq!(
                vm.verification_method_type(),
                &VerificationMethodType::Ed25519VerificationKey2020
            );
            assert_eq!(
                vm.public_key_field(),
                &PublicKeyField::Multibase {
                    public_key_multibase: "zH3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV"
                        .to_string()
                }
            );
        }
        {
            let vm = did_doc
                .dereference_key(&DidUrl::parse("#keys-2".to_string()).unwrap())
                .unwrap();
            assert_eq!(vm.id().to_string(), "did:example:123456789abcdefghi#keys-2");
        }
        {
            let vm = did_doc
                .dereference_key(
                    &DidUrl::parse("did:example:123456789abcdefghi#keys-3".to_string()).unwrap(),
                )
                .unwrap();
            assert_eq!(vm.id().to_string(), "#keys-3");
        }
        {
            let vm = did_doc
                .dereference_key(&DidUrl::parse("#keys-3".to_string()).unwrap())
                .unwrap();
            assert_eq!(vm.id().to_string(), "#keys-3");
        }
    }
}
