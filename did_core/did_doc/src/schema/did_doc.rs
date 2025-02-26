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
    pub fn new(id: Did) -> Self {
        DidDocument {
            id,
            also_known_as: vec![],
            controller: None,
            verification_method: vec![],
            authentication: vec![],
            assertion_method: vec![],
            key_agreement: vec![],
            capability_invocation: vec![],
            capability_delegation: vec![],
            service: vec![],
            extra: Default::default(),
        }
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

    pub fn verification_method_by_id(&self, id: &str) -> Option<&VerificationMethod> {
        self.verification_method
            .iter()
            .find(|vm| match vm.id().fragment() {
                Some(fragment) => fragment == id,
                None => false,
            })
    }

    pub fn authentication(&self) -> &[VerificationMethodKind] {
        self.authentication.as_ref()
    }

    fn try_match_and_deref_verification_method<'a>(
        &'a self,
        vm: &'a VerificationMethodKind,
        id: &str,
    ) -> Option<&'a VerificationMethod> {
        match vm {
            VerificationMethodKind::Resolved(vm) => {
                if vm.id().fragment() == Some(id) {
                    Some(vm)
                } else {
                    None
                }
            }
            VerificationMethodKind::Resolvable(vm_ref) => {
                if vm_ref.fragment() == Some(id) {
                    self.dereference_key(vm_ref)
                } else {
                    None
                }
            }
        }
    }

    pub fn authentication_by_id(&self, id: &str) -> Option<&VerificationMethod> {
        for vm in self.authentication.iter() {
            match self.try_match_and_deref_verification_method(vm, id) {
                Some(vm) => return Some(vm),
                None => continue,
            }
        }
        None
    }

    pub fn assertion_method(&self) -> &[VerificationMethodKind] {
        self.assertion_method.as_ref()
    }

    pub fn assertion_method_by_key(&self, id: &str) -> Option<&VerificationMethod> {
        for vm in self.assertion_method.iter() {
            match self.try_match_and_deref_verification_method(vm, id) {
                Some(vm) => return Some(vm),
                None => continue,
            }
        }
        None
    }

    pub fn key_agreement(&self) -> &[VerificationMethodKind] {
        self.key_agreement.as_ref()
    }

    pub fn key_agreement_by_id(&self, id: &str) -> Option<&VerificationMethod> {
        for vm in self.key_agreement.iter() {
            match self.try_match_and_deref_verification_method(vm, id) {
                Some(vm) => return Some(vm),
                None => continue,
            }
        }
        None
    }

    pub fn capability_invocation(&self) -> &[VerificationMethodKind] {
        self.capability_invocation.as_ref()
    }

    pub fn capability_invocation_by_id(&self, id: &str) -> Option<&VerificationMethod> {
        for vm in self.capability_invocation.iter() {
            match self.try_match_and_deref_verification_method(vm, id) {
                Some(vm) => return Some(vm),
                None => continue,
            }
        }
        None
    }

    pub fn capability_delegation(&self) -> &[VerificationMethodKind] {
        self.capability_delegation.as_ref()
    }

    pub fn capability_delegation_by_id(&self, id: &str) -> Option<&VerificationMethod> {
        for vm in self.capability_delegation.iter() {
            match self.try_match_and_deref_verification_method(vm, id) {
                Some(vm) => return Some(vm),
                None => continue,
            }
        }
        None
    }

    pub fn service(&self) -> &[Service] {
        self.service.as_ref()
    }

    pub fn extra_field(&self, key: &str) -> Option<&Value> {
        self.extra.get(key)
    }

    /// Scan the DIDDocument for a [VerificationMethod] that matches the given reference.
    pub fn dereference_key(&self, reference: &DidUrl) -> Option<&VerificationMethod> {
        let vms = self.verification_method.iter();

        // keys are typically in the VMs ^, but may be embedded in the other fields:
        let assertions = self.assertion_method.iter().filter_map(|k| k.resolved());
        let key_agreements = self.key_agreement.iter().filter_map(|k| k.resolved());
        let authentications = self.authentication.iter().filter_map(|k| k.resolved());
        let cap_invocations = self
            .capability_invocation
            .iter()
            .filter_map(|k| k.resolved());
        let cap_delegations = self
            .capability_delegation
            .iter()
            .filter_map(|k| k.resolved());

        let mut all_vms = vms
            .chain(assertions)
            .chain(key_agreements)
            .chain(authentications)
            .chain(cap_invocations)
            .chain(cap_delegations);

        all_vms.find(|vm| vm.id().fragment() == reference.fragment())
    }

    pub fn validate(&self) -> Result<(), DidDocumentBuilderError> {
        Ok(())
    }

    pub fn set_also_known_as(&mut self, uris: Vec<Uri>) {
        self.also_known_as = uris;
    }

    pub fn add_also_known_as(&mut self, uri: Uri) {
        self.also_known_as.push(uri);
    }

    pub fn set_controller(&mut self, controller: OneOrList<Did>) {
        self.controller = Some(controller);
    }

    pub fn add_verification_method(&mut self, method: VerificationMethod) {
        self.verification_method.push(method);
    }

    // authentication
    pub fn add_authentication(&mut self, method: VerificationMethodKind) {
        self.authentication.push(method);
    }

    pub fn add_authentication_object(&mut self, method: VerificationMethod) {
        self.authentication
            .push(VerificationMethodKind::Resolved(method));
    }

    pub fn add_authentication_ref(&mut self, reference: DidUrl) {
        self.authentication
            .push(VerificationMethodKind::Resolvable(reference));
    }

    // assertion
    pub fn add_assertion_method(&mut self, method: VerificationMethodKind) {
        self.assertion_method.push(method);
    }

    pub fn add_assertion_method_object(&mut self, method: VerificationMethod) {
        self.assertion_method
            .push(VerificationMethodKind::Resolved(method));
    }

    pub fn add_assertion_method_ref(&mut self, reference: DidUrl) {
        self.assertion_method
            .push(VerificationMethodKind::Resolvable(reference));
    }

    // key agreement
    pub fn add_key_agreement(&mut self, method: VerificationMethodKind) {
        self.key_agreement.push(method);
    }

    pub fn add_key_agreement_object(&mut self, method: VerificationMethod) {
        self.key_agreement
            .push(VerificationMethodKind::Resolved(method));
    }

    pub fn add_key_agreement_ref(&mut self, reference: DidUrl) {
        self.key_agreement
            .push(VerificationMethodKind::Resolvable(reference));
    }

    // capability invocation
    pub fn add_capability_invocation(&mut self, method: VerificationMethodKind) {
        self.capability_invocation.push(method);
    }

    pub fn add_capability_invocation_object(&mut self, method: VerificationMethod) {
        self.capability_invocation
            .push(VerificationMethodKind::Resolved(method));
    }

    pub fn add_capability_invocation_ref(&mut self, reference: DidUrl) {
        self.capability_invocation
            .push(VerificationMethodKind::Resolvable(reference));
    }

    // capability delegation
    pub fn add_capability_delegation(&mut self, method: VerificationMethodKind) {
        self.capability_delegation.push(method);
    }

    pub fn add_capability_delegation_object(&mut self, method: VerificationMethod) {
        self.capability_delegation
            .push(VerificationMethodKind::Resolved(method));
    }

    pub fn add_capability_delegation_ref(&mut self, reference: DidUrl) {
        self.capability_delegation
            .push(VerificationMethodKind::Resolvable(reference));
    }

    pub fn set_service(&mut self, services: Vec<Service>) {
        self.service = services;
    }

    pub fn add_service(&mut self, service: Service) {
        self.service.push(service);
    }

    pub fn set_extra_field(&mut self, key: String, value: Value) {
        self.extra.insert(key, value);
    }

    pub fn remove_extra_field(&mut self, key: &str) {
        self.extra.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::verification_method::{PublicKeyField, VerificationMethodType};

    #[test]
    fn test_construct_did_document() {
        let id = Did::parse("did:example:123456789abcdefghi".to_string()).unwrap();
        let also_known_as = Uri::new("https://example.com").unwrap();
        let controller = Did::parse("did:example:controller".to_string()).unwrap();

        let vm1_id = DidUrl::parse("did:example:vm1#vm1".to_string()).unwrap();
        let verification_method = VerificationMethod::builder()
            .id(vm1_id.clone())
            .controller(Did::parse("did:example:vm1".to_string()).unwrap())
            .verification_method_type(VerificationMethodType::Ed25519VerificationKey2018)
            .public_key(PublicKeyField::Base58 {
                public_key_base58: "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string(),
            })
            .build();

        let authentication_reference = DidUrl::parse("did:example:authref".to_string()).unwrap();
        let assertion_method = VerificationMethod::builder()
            .id(DidUrl::parse("did:example:am1".to_string()).unwrap())
            .controller(Did::parse("did:example:am2".to_string()).unwrap())
            .verification_method_type(VerificationMethodType::Ed25519VerificationKey2018)
            .public_key(PublicKeyField::Base58 {
                public_key_base58: "H3C2AVvLMv6gmMNam3uVAjZpfkcJCwDwnZn6z3wXmqPV".to_string(),
            })
            .build();

        let service_id = Uri::new("did:example:123456789abcdefghi;service-1").unwrap();
        let service_endpoint = "https://example.com/service";
        let service = Service::new(
            service_id,
            service_endpoint.try_into().unwrap(),
            OneOrList::One(ServiceType::Other("test-service".to_string())),
            HashMap::default(),
        );

        let mut did_doc = DidDocument::new(id.clone());
        did_doc.set_also_known_as(vec![also_known_as.clone()]);
        did_doc.set_controller(OneOrList::One(controller.clone()));
        did_doc.add_verification_method(verification_method.clone());

        did_doc.add_authentication_object(verification_method.clone());
        did_doc.add_authentication_ref(authentication_reference.clone());

        did_doc.add_assertion_method_object(assertion_method.clone());
        did_doc.add_assertion_method_ref(authentication_reference.clone());

        did_doc.add_key_agreement_object(verification_method.clone());
        did_doc.add_key_agreement_ref(authentication_reference.clone());

        did_doc.add_capability_invocation_object(verification_method.clone());
        did_doc.add_capability_invocation_ref(authentication_reference.clone());

        did_doc.add_capability_delegation_object(verification_method.clone());
        did_doc.add_capability_delegation_ref(authentication_reference.clone());

        did_doc.set_service(vec![service.clone()])
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
        let vm1 = VerificationMethod::builder()
            .id(vm1_id)
            .controller(controller.clone())
            .verification_method_type(VerificationMethodType::JsonWebKey2020)
            .public_key(PublicKeyField::Jwk {
                public_key_jwk: JsonWebKey::from_str(
                    r#"{
                "kty": "EC",
                "crv": "BLS12381_G1",
                "x": "hxF12gtsn9ju4-kJq2-nUjZQKVVWpcBAYX5VHnUZMDilClZsGuOaDjlXS8pFE1GG"
                }"#,
                )
                .unwrap(),
            })
            .build();

        let vm2_id = DidUrl::parse("#g2".to_string()).unwrap();
        let vm2 = VerificationMethod::builder()
            .id(vm2_id)
            .controller(controller.clone())
            .verification_method_type(VerificationMethodType::JsonWebKey2020)
            .public_key(PublicKeyField::Jwk { public_key_jwk: JsonWebKey::from_str(
                r#"{
                "kty": "EC",
                "crv": "BLS12381_G2",
                "x": "l4MeBsn_OGa2OEDtHeHdq0TBC8sYh6QwoI7QsNtZk9oAru1OnGClaAPlMbvvs73EABDB6GjjzybbOHarkBmP6pon8H1VuMna0nkEYihZi8OodgdbwReDiDvWzZuXXMl-"
            }"#,
            ).unwrap()})
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
        let ka1 = VerificationMethod::builder()
            .id(ka1_id)
            .controller(controller.clone())
            .verification_method_type(VerificationMethodType::X25519KeyAgreementKey2019)
            .public_key(PublicKeyField::Base58 {
                public_key_base58: "CaSHXEvLKS6SfN9aBfkVGBpp15jSnaHazqHgLHp8KZ3Y".to_string(),
            })
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
