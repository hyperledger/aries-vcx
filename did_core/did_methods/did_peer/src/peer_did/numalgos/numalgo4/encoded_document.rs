use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use did_doc::schema::service::Service;
use did_doc::schema::types::uri::Uri;
use did_doc::schema::verification_method::{PublicKeyField, VerificationMethodType};
use did_parser::{Did, DidUrl};
use display_as_json::Display;

/// The following DidPeer4* structs are similar to those defined in did_doc crate,
/// however with minor differences defined by https://identity.foundation/peer-did-method-spec/#creating-a-did
/// In nutshell:
/// - The document MUST NOT include an id at the root.
/// - All identifiers within this document MUST be relative
/// - All references pointing to resources within this document MUST be relative
/// - For verification methods, the controller MUST be omitted if the controller is the document owner.
///
/// These structures are **only** used for construction of did:peer:4 DIDs
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, Display, derive_builder::Builder)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
#[builder(default)]
pub struct DidPeer4EncodedDocument {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    also_known_as: Vec<Uri>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    verification_method: Vec<DidPeer4VerificationMethodKind>,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum DidPeer4VerificationMethodKind {
    Resolved(DidPeer4VerificationMethod),
    Resolvable(DidUrl), // must be relative, can we break down DidUrl into new type RelativeDidUrl?
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DidPeer4VerificationMethod {
    id: DidUrl,
    // must be relative, can we break down DidUrl into new type RelativeDidUrl?
    controller: Option<Did>,
    #[serde(rename = "type")]
    verification_method_type: VerificationMethodType,
    #[serde(flatten)]
    public_key: PublicKeyField,
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use did_doc::schema::service::Service;
    use did_doc::schema::service::typed::ServiceType;
    use did_doc::schema::types::uri::Uri;
    use did_doc::schema::utils::OneOrList;

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
            .service(vec!(service))
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
