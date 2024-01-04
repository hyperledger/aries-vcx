use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

use crate::{
    error::DidDocumentBuilderError,
    schema::{
        did_doc::DidDocument,
        verification_method::{VerificationMethod, VerificationMethodKind, VerificationMethodType},
    },
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum OneOrList<T> {
    One(T),
    List(Vec<T>),
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
    pub fn find_key_agreement_of_type(
        &self,
        key_types: &[VerificationMethodType],
    ) -> Result<VerificationMethod, DidDocumentBuilderError> {
        for verification_method_kind in self.key_agreement() {
            let verification_method = match verification_method_kind {
                VerificationMethodKind::Resolved(verification_method) => verification_method,
                VerificationMethodKind::Resolvable(reference) => {
                    match self.dereference_key(reference) {
                        None => {
                            return Err(DidDocumentBuilderError::CustomError(format!(
                                "Unable to dereference key: {}",
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
        Err(DidDocumentBuilderError::CustomError(
            "No supported key_agreement keys have been found".to_string(),
        ))
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
        let methods = &vec![
            VerificationMethodType::Ed25519VerificationKey2020,
            VerificationMethodType::X25519KeyAgreementKey2020,
        ];
        let key = did_document.find_key_agreement_of_type(methods).unwrap();
        assert_eq!(key.id().to_string(), "#bar")
    }

    #[test]
    fn should_not_resolve_key_agreement() {
        let did_document: DidDocument = serde_json::from_str(DID_DOC).unwrap();
        let methods = &vec![VerificationMethodType::Bls12381G1Key2020];
        let err = did_document
            .find_key_agreement_of_type(methods)
            .expect_err("expected error");
        assert!(err
            .to_string()
            .contains("No supported key_agreement keys have been found"))
    }
}
