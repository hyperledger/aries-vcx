use std::collections::HashMap;

use did_doc::schema::{
    did_doc::DidDocument,
    service::Service,
    types::uri::Uri,
    verification_method::{
        PublicKeyField, VerificationMethod, VerificationMethodKind, VerificationMethodType,
    },
};
use did_parser_nom::DidUrl;
use display_as_json::Display;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use typed_builder::TypedBuilder;

use crate::peer_did::{numalgos::numalgo4::Numalgo4, PeerDid};

/// The following structs DidPeer4ConstructionDidDoc, DidPeer4VerificationMethodKind are similar to
/// those defined in did_doc crate, however with minor differences defined
/// in https://identity.foundation/peer-did-method-spec/#creating-a-did
/// In nutshell:
/// - The document MUST NOT include an id at the root.
/// - All identifiers within this document MUST be relative
/// - All references pointing to resources within this document MUST be relative
/// - For verification methods, the controller MUST be omitted if the controller is the document
///   owner.
///
/// These structures are **only** used for construction of did:peer:4 DIDs
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, Display)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct DidPeer4ConstructionDidDocument {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    also_known_as: Vec<Uri>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    verification_method: Vec<DidPeer4VerificationMethod>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    authentication: Vec<DidPeer4VerificationMethodKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    assertion_method: Vec<DidPeer4VerificationMethodKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    key_agreement: Vec<DidPeer4VerificationMethodKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    capability_invocation: Vec<DidPeer4VerificationMethodKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    capability_delegation: Vec<DidPeer4VerificationMethodKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    service: Vec<Service>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

impl DidPeer4ConstructionDidDocument {
    pub fn new() -> DidPeer4ConstructionDidDocument {
        DidPeer4ConstructionDidDocument {
            also_known_as: vec![],
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

    // - Performs DidDocument "contextualization" as described here: https://identity.foundation/peer-did-method-spec/#resolving-a-did
    pub(crate) fn contextualize_to_did_doc(&self, did_peer_4: &PeerDid<Numalgo4>) -> DidDocument {
        let did_doc_id = did_peer_4.did().clone();
        let mut did_doc = DidDocument::new(did_doc_id);
        did_doc.set_service(self.service.clone());
        for vm in &self.verification_method {
            did_doc.add_verification_method(vm.contextualize(did_peer_4));
        }
        for vm in &self.key_agreement {
            did_doc.add_key_agreement(vm.contextualize(did_peer_4))
        }
        for vm in &self.authentication {
            did_doc.add_authentication(vm.contextualize(did_peer_4))
        }
        for vm in &self.assertion_method {
            did_doc.add_assertion_method(vm.contextualize(did_peer_4))
        }
        for vm in &self.capability_delegation {
            did_doc.add_capability_delegation(vm.contextualize(did_peer_4))
        }
        for vm in &self.capability_invocation {
            did_doc.add_capability_invocation(vm.contextualize(did_peer_4))
        }
        // ** safety note (panic) **
        // Formally every DID is URI. Assuming the parsers for both DID and URI correctly
        // implement   respective specs, this will never panic.
        let did_short_form = did_peer_4.short_form().to_string();
        let did_as_uri = Uri::new(&did_short_form).unwrap_or_else(|_| {
            panic!(
                "DID or URI implementation is buggy, because DID {} failed to be parsed as URI. \
                 This counters W3C DID-CORE spec which states that \"DIDs are URIs\" [RFC3986].",
                did_short_form
            )
        });
        did_doc.add_also_known_as(did_as_uri);
        did_doc
    }

    pub fn set_also_known_as(&mut self, uris: Vec<Uri>) {
        self.also_known_as = uris;
    }

    pub fn add_also_known_as(&mut self, uri: Uri) {
        self.also_known_as.push(uri);
    }

    pub fn add_verification_method(&mut self, method: DidPeer4VerificationMethod) {
        self.verification_method.push(method);
    }

    pub fn add_authentication(&mut self, method: DidPeer4VerificationMethod) {
        self.authentication
            .push(DidPeer4VerificationMethodKind::Resolved(method));
    }

    pub fn add_authentication_ref(&mut self, reference: DidUrl) {
        self.authentication
            .push(DidPeer4VerificationMethodKind::Resolvable(reference));
    }

    pub fn add_assertion_method(&mut self, method: DidPeer4VerificationMethod) {
        self.assertion_method
            .push(DidPeer4VerificationMethodKind::Resolved(method));
    }

    pub fn add_assertion_method_ref(&mut self, reference: DidUrl) {
        self.assertion_method
            .push(DidPeer4VerificationMethodKind::Resolvable(reference));
    }

    pub fn add_key_agreement(&mut self, method: DidPeer4VerificationMethod) {
        self.key_agreement
            .push(DidPeer4VerificationMethodKind::Resolved(method));
    }

    pub fn add_key_agreement_ref(&mut self, refernece: DidUrl) {
        self.key_agreement
            .push(DidPeer4VerificationMethodKind::Resolvable(refernece));
    }

    pub fn add_capability_invocation(&mut self, method: DidPeer4VerificationMethod) {
        self.capability_invocation
            .push(DidPeer4VerificationMethodKind::Resolved(method));
    }

    pub fn add_capability_invocation_ref(&mut self, reference: DidUrl) {
        self.capability_invocation
            .push(DidPeer4VerificationMethodKind::Resolvable(reference));
    }

    pub fn add_capability_delegation(&mut self, method: DidPeer4VerificationMethod) {
        self.capability_delegation
            .push(DidPeer4VerificationMethodKind::Resolved(method));
    }

    pub fn add_capability_delegation_ref(&mut self, reference: DidUrl) {
        self.capability_delegation
            .push(DidPeer4VerificationMethodKind::Resolvable(reference));
    }

    // Setter for `service`
    pub fn set_service(&mut self, services: Vec<Service>) {
        self.service = services;
    }

    // Add a service
    pub fn add_service(&mut self, service: Service) {
        self.service.push(service);
    }

    // Setter for an extra field
    pub fn set_extra_field(&mut self, key: String, value: Value) {
        self.extra.insert(key, value);
    }

    // Remove an extra field
    pub fn remove_extra_field(&mut self, key: &str) {
        self.extra.remove(key);
    }
}

/// Struct `DidPeer4VerificationMethodKind` is quite similar to `VerificationMethodKind` (defined
/// in did_doc crate) utilized by `DidPeer4ConstructionDidDoc` for construction of did:peer:4* DIDs.
/// The spec describes differences: https://identity.foundation/peer-did-method-spec/#creating-a-did
/// The spec is describing did document in general, but the restrictions applies
/// to Verification Methods such that:
/// - Verification Method IDs MUST be relative
/// - Reference to Verification Methods must be relative
/// - Controller MUST be omitted if the controller is the document owner.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)] // todo: revisit this
pub enum DidPeer4VerificationMethodKind {
    Resolved(DidPeer4VerificationMethod),
    Resolvable(DidUrl), /* MUST be relative,
                        TODO: Should we have subtype such as RelativeDidUrl? */
}

impl DidPeer4VerificationMethodKind {
    pub fn contextualize(&self, did_peer_4: &PeerDid<Numalgo4>) -> VerificationMethodKind {
        match self {
            DidPeer4VerificationMethodKind::Resolved(vm) => {
                VerificationMethodKind::Resolved(vm.contextualize(did_peer_4))
            }
            DidPeer4VerificationMethodKind::Resolvable(did_url) => {
                VerificationMethodKind::Resolvable(did_url.clone())
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, TypedBuilder)]
#[serde(rename_all = "camelCase")]
pub struct DidPeer4VerificationMethod {
    id: DidUrl,
    // - Controller MUST be relative, can we break down DidUrl into new type RelativeDidUrl?
    // - Controller MUST be omitted, if the controller is the document owner (main reason why this
    //   is different from did_doc::schema::verification_method::VerificationMethod)
    // - TODO: add support for controller different than the document owner (how does that work for
    //   peer DIDs?)
    // pub(crate) controller: Option<Did>,
    #[serde(rename = "type")]
    verification_method_type: VerificationMethodType,
    #[serde(flatten)]
    public_key: PublicKeyField,
}

impl DidPeer4VerificationMethod {
    pub(crate) fn contextualize(&self, did_peer_4: &PeerDid<Numalgo4>) -> VerificationMethod {
        VerificationMethod::builder()
            .id(self.id.clone())
            .controller(did_peer_4.short_form().clone())
            .verification_method_type(self.verification_method_type)
            .public_key(self.public_key.clone())
            .build()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use did_doc::schema::{
        service::{typed::ServiceType, Service},
        types::uri::Uri,
        utils::OneOrList,
    };

    use crate::peer_did::numalgos::numalgo4::construction_did_doc::DidPeer4ConstructionDidDocument;

    #[test]
    fn test_encoded_document_has_builder_api() {
        let service = Service::new(
            Uri::new("#service-0").unwrap(),
            "https://example.com/endpoint".parse().unwrap(),
            OneOrList::One(ServiceType::DIDCommV2),
            HashMap::default(),
        );
        let mut construction_did_doc = DidPeer4ConstructionDidDocument::new();
        construction_did_doc.add_service(service);
        assert_eq!(construction_did_doc.service.len(), 1);
        assert_eq!(construction_did_doc.assertion_method.len(), 0);
        assert_eq!(construction_did_doc.authentication.len(), 0);
        assert_eq!(construction_did_doc.key_agreement.len(), 0);
        assert_eq!(construction_did_doc.capability_invocation.len(), 0);
        assert_eq!(construction_did_doc.capability_delegation.len(), 0);
        assert_eq!(construction_did_doc.verification_method.len(), 0);
        assert_eq!(construction_did_doc.also_known_as.len(), 0);
        assert_eq!(construction_did_doc.extra.len(), 0);
    }
}
