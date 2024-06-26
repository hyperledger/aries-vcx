use std::cmp::Ordering;

use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use did_doc::schema::{
    did_doc::DidDocument,
    verification_method::{VerificationMethod, VerificationMethodKind},
};
use public_key::Key;

use crate::{
    error::DidPeerError,
    peer_did::numalgos::numalgo2::{
        purpose::ElementPurpose,
        service_abbreviation::{abbreviate_service, ServiceAbbreviatedDidPeer2},
        verification_method::get_key_by_verification_method,
    },
};

pub(crate) fn append_encoded_key_segments(
    mut did: String,
    did_document: &DidDocument,
) -> Result<String, DidPeerError> {
    for am in did_document.assertion_method() {
        did = append_encoded_key_segment(did, did_document, am, ElementPurpose::Assertion)?;
    }
    for ka in did_document.key_agreement() {
        did = append_encoded_key_segment(did, did_document, ka, ElementPurpose::Encryption)?;
    }
    for a in did_document.authentication() {
        did = append_encoded_key_segment(did, did_document, a, ElementPurpose::Verification)?;
    }
    for ci in did_document.capability_invocation() {
        did = append_encoded_key_segment(
            did,
            did_document,
            ci,
            ElementPurpose::CapabilityInvocation,
        )?;
    }
    for cd in did_document.capability_delegation() {
        did = append_encoded_key_segment(
            did,
            did_document,
            cd,
            ElementPurpose::CapabilityDelegation,
        )?;
    }
    Ok(did)
}

pub(crate) fn append_encoded_service_segment(
    mut did: String,
    did_document: &DidDocument,
) -> Result<String, DidPeerError> {
    let services_abbreviated = did_document
        .service()
        .iter()
        .map(abbreviate_service)
        .collect::<Result<Vec<ServiceAbbreviatedDidPeer2>, _>>()?;

    let service_encoded = match services_abbreviated.len().cmp(&1) {
        Ordering::Less => None,
        Ordering::Equal => {
            let service_abbreviated = services_abbreviated.first().unwrap();
            Some(STANDARD_NO_PAD.encode(serde_json::to_vec(&service_abbreviated)?))
        }
        Ordering::Greater => {
            // todo: Easy fix; this should be implemented by iterating over each service and then
            //       appending the services in peer did, separated by a dot.
            //       See https://identity.foundation/peer-did-method-spec/
            unimplemented!("Multiple services are not supported yet")
        }
    };

    if let Some(service_encoded) = service_encoded {
        let encoded = format!(".{}{}", ElementPurpose::Service, service_encoded);
        did.push_str(&encoded);
    }

    Ok(did)
}

fn append_encoded_key_segment(
    did: String,
    did_document: &DidDocument,
    vm: &VerificationMethodKind,
    purpose: ElementPurpose,
) -> Result<String, DidPeerError> {
    let vm = resolve_verification_method(did_document, vm)?;
    let key = get_key_by_verification_method(vm)?;
    Ok(append_key_to_did(did, key, purpose))
}

fn resolve_verification_method<'a>(
    did_document: &'a DidDocument,
    vm: &'a VerificationMethodKind,
) -> Result<&'a VerificationMethod, DidPeerError> {
    match vm {
        VerificationMethodKind::Resolved(vm) => Ok(vm),
        VerificationMethodKind::Resolvable(did_url) => {
            did_document
                .dereference_key(did_url)
                .ok_or(DidPeerError::InvalidKeyReference(format!(
                    "Could not resolve verification method: {} on DID document: {}",
                    did_url, did_document
                )))
        }
    }
}

fn append_key_to_did(mut did: String, key: Key, purpose: ElementPurpose) -> String {
    let encoded = format!(".{}{}", purpose, key.fingerprint());
    did.push_str(&encoded);
    did
}

#[cfg(test)]
mod tests {
    use did_doc::schema::{
        service::{
            service_key_kind::ServiceKeyKind,
            typed::{didcommv2::ExtraFieldsDidCommV2, ServiceType},
            Service,
        },
        types::uri::Uri,
        utils::OneOrList,
        verification_method::{PublicKeyField, VerificationMethod, VerificationMethodType},
    };
    use did_parser_nom::{Did, DidUrl};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        helpers::convert_to_hashmap,
        peer_did::{numalgos::numalgo2::Numalgo2, PeerDid},
        resolver::options::PublicKeyEncoding,
    };

    fn create_verification_method(
        verification_method_id: String,
        controller_did: String,
        key: String,
        verification_type: VerificationMethodType,
    ) -> VerificationMethod {
        VerificationMethod::builder()
            .id(verification_method_id.parse().unwrap())
            .controller(Did::parse(controller_did).unwrap())
            .verification_method_type(verification_type)
            .public_key(PublicKeyField::Multibase {
                public_key_multibase: key,
            })
            .build()
    }

    #[test]
    fn test_append_encoded_key_segments() {
        let did = "did:peer:2";
        let key_0 = "z6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc";
        let key_1 = "z6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V";
        let did_full = format!("{}.E{}.V{}", did, key_0, key_1);

        let vm_0 = create_verification_method(
            "#key-1".to_string(),
            did_full.to_string(),
            key_0.to_string(),
            VerificationMethodType::X25519KeyAgreementKey2020,
        );
        let vm_1 = create_verification_method(
            "#key-2".to_string(),
            did_full.to_string(),
            key_1.to_string(),
            VerificationMethodType::Ed25519VerificationKey2020,
        );

        let mut did_document = DidDocument::new(Did::parse(did_full.clone()).unwrap());
        did_document.add_key_agreement_ref(vm_0.id().to_owned());
        did_document.add_verification_method(vm_0);
        did_document.add_authentication_ref(vm_1.id().to_owned());
        did_document.add_verification_method(vm_1);

        let did = append_encoded_key_segments(did.to_string(), &did_document).unwrap();
        assert_eq!(did, did_full);
    }

    #[tokio::test]
    async fn test_append_encoded_service_segment() {
        let did = "did:peer:2";
        let service = "eyJpZCI6IiNzZXJ2aWNlLTAiLCJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXSwiYSI6WyJkaWRjb21tL3YyIiwiZGlkY29tbS9haXAyO2Vudj1yZmM1ODciXX0";
        let did_expected = format!("{}.S{}", did, service);

        let extra = ExtraFieldsDidCommV2::builder()
            .routing_keys(vec![ServiceKeyKind::Reference(
                "did:example:somemediator#somekey".parse().unwrap(),
            )])
            .accept(vec!["didcomm/v2".into(), "didcomm/aip2;env=rfc587".into()])
            .build();

        let service = Service::new(
            Uri::new("#service-0").unwrap(),
            "https://example.com/endpoint".parse().unwrap(),
            OneOrList::One(ServiceType::DIDCommV2),
            convert_to_hashmap(&extra).unwrap(),
        );

        let mut did_document = DidDocument::new(did_expected.parse().unwrap());
        did_document.add_service(service);

        let did = append_encoded_service_segment(did.to_string(), &did_document).unwrap();

        let did_parsed = PeerDid::<Numalgo2>::parse(did.clone()).unwrap();
        let ddo = did_parsed
            .to_did_doc_builder(PublicKeyEncoding::Base58)
            .unwrap();

        let did_expected_parsed = PeerDid::<Numalgo2>::parse(did_expected.clone()).unwrap();
        let ddo_expected = did_expected_parsed
            .to_did_doc_builder(PublicKeyEncoding::Base58)
            .unwrap();

        assert_eq!(ddo, ddo_expected);
        assert_eq!(did, did_expected);
    }

    #[test]
    fn test_append_encoded_segments_error() {
        let did = "did:peer:2";
        let key = "invalid_key";
        let did_full = format!("{}.E{}", did, key);

        let vm = create_verification_method(
            "#key-1".to_string(),
            did_full.to_string(),
            key.to_string(),
            VerificationMethodType::X25519KeyAgreementKey2020,
        );

        let mut did_document = DidDocument::new(did_full.parse().unwrap());
        did_document.add_key_agreement_object(vm);

        let result = append_encoded_key_segments(did.to_string(), &did_document);
        assert!(result.is_err());
    }

    #[test]
    fn test_append_encoded_key_segments_multiple_keys() {
        let did = "did:peer:2";
        let key_0 = "z6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc";
        let key_1 = "z6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V";
        let key_2 = "z6Mkumaf3DZPAw8CN8r7vqA4UbW5b6hFfpq6nM4xud1MBZ9n";
        let did_full = format!("{}.A{}.E{}.V{}", did, key_0, key_1, key_2);

        let vm_0 = create_verification_method(
            "#key-1".to_string(),
            did_full.to_string(),
            key_0.to_string(),
            VerificationMethodType::X25519KeyAgreementKey2020,
        );
        let vm_1 = create_verification_method(
            "#key-2".to_string(),
            did_full.to_string(),
            key_1.to_string(),
            VerificationMethodType::Ed25519VerificationKey2020,
        );
        let vm_2 = create_verification_method(
            "#key-3".to_string(),
            did_full.to_string(),
            key_2.to_string(),
            VerificationMethodType::Ed25519VerificationKey2020,
        );

        let mut did_document = DidDocument::new(did_full.parse().unwrap());
        did_document.add_assertion_method_object(vm_0);
        did_document.add_key_agreement_object(vm_1);
        did_document.add_authentication_object(vm_2);

        let did = append_encoded_key_segments(did.to_string(), &did_document).unwrap();
        assert_eq!(did, did_full);
    }

    #[test]
    fn test_append_encoded_key_segments_resolvable_key() {
        let env = env_logger::Env::default().default_filter_or("info");
        env_logger::init_from_env(env);

        let did = "did:peer:2";
        let key = "z6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc";
        let did_full = format!("{}.E{}.V{}", did, key, key);
        let reference = "key-1";

        let vm = create_verification_method(
            format!("#{}", reference),
            did_full.to_string(),
            key.to_string(),
            VerificationMethodType::X25519KeyAgreementKey2020,
        );

        let mut did_document = DidDocument::new(did_full.parse().unwrap());
        did_document.add_verification_method(vm);
        did_document.add_authentication_ref(DidUrl::from_fragment(reference.to_string()).unwrap());
        did_document.add_key_agreement_ref(DidUrl::from_fragment(reference.to_string()).unwrap());

        let did = append_encoded_key_segments(did.to_string(), &did_document).unwrap();
        assert_eq!(did, did_full);
    }
}
