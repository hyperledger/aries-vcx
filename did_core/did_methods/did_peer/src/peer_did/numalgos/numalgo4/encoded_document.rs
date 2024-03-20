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

use crate::peer_did::{numalgos::numalgo4::Numalgo4, PeerDid};

/// The following DidPeer4* structs are similar to those defined in did_doc crate,
/// however with minor differences defined by https://identity.foundation/peer-did-method-spec/#creating-a-did
/// In nutshell:
/// - The document MUST NOT include an id at the root.
/// - All identifiers within this document MUST be relative
/// - All references pointing to resources within this document MUST be relative
/// - For verification methods, the controller MUST be omitted if the controller is the document
///   owner.
///
/// These structures are **only** used for construction of did:peer:4 DIDs
#[derive(
    Serialize, Deserialize, Clone, Debug, PartialEq, Default, Display, derive_builder::Builder,
)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
#[builder(default)]
pub struct DidPeer4EncodedDocument {
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

impl DidPeer4EncodedDocument {
    // - Performs DidDocument "contextualization" as described here: https://identity.foundation/peer-did-method-spec/#resolving-a-did
    pub(crate) fn contextualize_to_did_doc(&self, did_peer_4: &PeerDid<Numalgo4>) -> DidDocument {
        let mut builder =
            DidDocument::builder(did_peer_4.did().clone()).set_service(self.service.clone());
        for vm in &self.verification_method {
            builder.add_verification_method_2(vm.contextualize(did_peer_4));
        }
        builder.build()
    }
}

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

// TODO: use builder instead of pub(crate) ?
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DidPeer4VerificationMethod {
    pub(crate) id: DidUrl,
    // - Controller MUST be relative, can we break down DidUrl into new type RelativeDidUrl?
    // - Controller MUST be omitted, if the controller is the document owner (main reason why this
    //   is different from did_doc::schema::verification_method::VerificationMethod)
    // - TODO: add support for controller different than the document owner (how does that work for
    //   peer DIDs?)
    // controller: Option<Did>,
    #[serde(rename = "type")]
    pub(crate) verification_method_type: VerificationMethodType,
    #[serde(flatten)]
    pub(crate) public_key: PublicKeyField,
}

impl DidPeer4VerificationMethod {
    pub(crate) fn contextualize(&self, did_peer_4: &PeerDid<Numalgo4>) -> VerificationMethod {
        VerificationMethod::builder(
            self.id.clone(),
            did_peer_4.did().clone(),
            self.verification_method_type,
        )
        .add_public_key_field(self.public_key.clone())
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

    use crate::peer_did::numalgos::numalgo4::encoded_document::DidPeer4EncodedDocumentBuilder;

    #[test]
    fn test_encoded_document_has_builder_api() {
        let service = Service::new(
            Uri::new("#service-0").unwrap(),
            "https://example.com/endpoint".parse().unwrap(),
            OneOrList::One(ServiceType::DIDCommV2),
            HashMap::default(),
        );
        let encoded_document = DidPeer4EncodedDocumentBuilder::default()
            .service(vec![service])
            .build()
            .unwrap();
        assert_eq!(encoded_document.service.len(), 1);
        assert_eq!(encoded_document.assertion_method.len(), 0);
        assert_eq!(encoded_document.authentication.len(), 0);
        assert_eq!(encoded_document.key_agreement.len(), 0);
        assert_eq!(encoded_document.capability_invocation.len(), 0);
        assert_eq!(encoded_document.capability_delegation.len(), 0);
        assert_eq!(encoded_document.verification_method.len(), 0);
        assert_eq!(encoded_document.also_known_as.len(), 0);
        assert_eq!(encoded_document.extra.len(), 0);
    }
}
