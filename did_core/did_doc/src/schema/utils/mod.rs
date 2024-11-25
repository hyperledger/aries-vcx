pub mod error;

use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

use crate::schema::{
    did_doc::DidDocument,
    service::{typed::ServiceType, Service},
    types::uri::Uri,
    utils::error::DidDocumentLookupError,
    verification_method::{VerificationMethod, VerificationMethodKind, VerificationMethodType},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum OneOrList<T> {
    One(T),
    List(Vec<T>),
}

impl<T> From<Vec<T>> for OneOrList<T> {
    fn from(mut value: Vec<T>) -> Self {
        match value.len() {
            1 => OneOrList::One(value.remove(0)),
            _ => OneOrList::List(value),
        }
    }
}

impl OneOrList<String> {
    pub fn first(&self) -> Option<String> {
        match self {
            OneOrList::One(s) => Some(s.clone()),
            OneOrList::List(s) => s.first().cloned(),
        }
    }
}

impl<T: Display + Debug> Display for OneOrList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OneOrList::One(t) => write!(f, "{}", t),
            OneOrList::List(t) => write!(f, "{:?}", t),
        }
    }
}

impl DidDocument {
    pub fn get_key_agreement_of_type(
        &self,
        key_types: &[VerificationMethodType],
    ) -> Result<VerificationMethod, DidDocumentLookupError> {
        for verification_method_kind in self.key_agreement() {
            let verification_method = match verification_method_kind {
                VerificationMethodKind::Resolved(verification_method) => verification_method,
                VerificationMethodKind::Resolvable(reference) => {
                    match self.dereference_key(reference) {
                        None => {
                            return Err(DidDocumentLookupError::new(format!(
                                "Unable to resolve key agreement key by reference: {}",
                                reference
                            )))
                        }
                        Some(verification_method) => verification_method,
                    }
                }
            };
            for key_type in key_types {
                if verification_method.verification_method_type() == key_type {
                    return Ok(verification_method.clone());
                }
            }
        }
        Err(DidDocumentLookupError::new(
            "No supported key_agreement keys have been found".to_string(),
        ))
    }

    pub fn get_service_of_type(
        &self,
        service_type: &ServiceType,
    ) -> Result<Service, DidDocumentLookupError> {
        self.service()
            .iter()
            .find(|service| service.service_types().contains(service_type))
            .cloned()
            .ok_or(DidDocumentLookupError::new(format!(
                "Failed to look up service object by type {}",
                service_type
            )))
    }

    pub fn get_service_by_id(&self, id: &Uri) -> Result<Service, DidDocumentLookupError> {
        self.service()
            .iter()
            .find(|service| service.id() == id)
            .cloned()
            .ok_or(DidDocumentLookupError::new(format!(
                "Failed to look up service object by id {}",
                id
            )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::verification_method::VerificationMethodType;

    const DID_DOC: &str = r##"
    {
      "@context": [
        "https://w3.org/ns/did/v1",
        "https://w3id.org/security/suites/ed25519-2018/v1"
      ],
      "id": "did:web:did-actor-alice",
      "alsoKnownAs": [
          "https://example.com/user-profile/123"
      ],
      "keyAgreement": [
        {
          "id": "#foo",
          "type": "Bls12381G2Key2020",
          "controller": "did:web:did-actor-alice",
          "publicKeyBase58": "CaSHXEvLKS6SfN9aBfkVGBpp15jSnaHazqHgLHp8KZ3Y"
        },
        {
          "id": "#bar",
          "type": "X25519KeyAgreementKey2020",
          "controller": "did:web:did-actor-alice",
          "publicKeyBase58": "CaSHXEvLKS6SfN9aBfkVGBpp15jSnaHazqHgLHp8KZ3Y"
        }
      ]
    }
    "##;

    #[test]
    fn should_resolve_key_agreement() {
        let did_document: DidDocument = serde_json::from_str(DID_DOC).unwrap();
        let methods = &[
            VerificationMethodType::Ed25519VerificationKey2020,
            VerificationMethodType::X25519KeyAgreementKey2020,
        ];
        let key = did_document.get_key_agreement_of_type(methods).unwrap();
        assert_eq!(key.id().to_string(), "#bar")
    }

    #[test]
    fn should_not_resolve_key_agreement() {
        let did_document: DidDocument = serde_json::from_str(DID_DOC).unwrap();
        let methods = &[VerificationMethodType::Bls12381G1Key2020];
        let err = did_document
            .get_key_agreement_of_type(methods)
            .expect_err("expected error");
        assert!(err
            .to_string()
            .contains("No supported key_agreement keys have been found"))
    }
}
