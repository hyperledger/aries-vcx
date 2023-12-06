use std::cmp::Ordering;

use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use did_doc::schema::{
    did_doc::DidDocument,
    service::{
        extra_fields::{ServiceAcceptType, ServiceKeyKind},
        Service,
    },
    utils::OneOrList,
    verification_method::{VerificationMethod, VerificationMethodKind},
};
use public_key::Key;
use serde_json::from_value;

use crate::{
    error::DidPeerError,
    peer_did::numalgos::numalgo2::{
        purpose::ElementPurpose, service_abbreviated::ServiceAbbreviatedDidPeer2,
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

    for vm in did_document.verification_method() {
        did = append_encoded_key_segment(
            did,
            did_document,
            &VerificationMethodKind::Resolved(vm.to_owned()),
            ElementPurpose::Verification,
        )?;
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
        VerificationMethodKind::Resolvable(did_url) => did_document
            .dereference_key(did_url)
            .ok_or(DidPeerError::InvalidKeyReference(did_url.to_string())),
    }
}

fn append_key_to_did(mut did: String, key: Key, purpose: ElementPurpose) -> String {
    let encoded = format!(".{}{}", purpose, key.fingerprint());
    did.push_str(&encoded);
    did
}

fn abbreviate_service(service: &Service) -> Result<ServiceAbbreviatedDidPeer2, DidPeerError> {
    let service_endpoint = service.service_endpoint().clone();
    let routing_keys = {
        service
            .extra()
            .get("routingKeys")
            .map(|value| {
                from_value::<Vec<ServiceKeyKind>>(value.clone()).map_err(|_| {
                    DidPeerError::ParsingError(format!(
                        "Could not parse routing keys as Vector of Strings. Value of \
                         routing_keys: {}",
                        value
                    ))
                })
            })
            .unwrap_or_else(|| Ok(vec![]))
    }?;
    let accept = {
        service
            .extra()
            .get("accept")
            .map(|value| {
                from_value::<Vec<ServiceAcceptType>>(value.clone()).map_err(|_| {
                    DidPeerError::ParsingError(format!(
                        "Could not parse accept as Vector of Strings. Value of accept: {}",
                        value
                    ))
                })
            })
            .unwrap_or_else(|| Ok(vec![]))
    }?;
    let mut service_type = service.service_type().clone();
    service_type = match service_type {
        OneOrList::List(mut service_types) => {
            service_types.iter_mut().for_each(|value| {
                if value == "DIDCommMessaging" {
                    *value = "dm".to_string();
                }
            });
            OneOrList::List(service_types)
        }
        OneOrList::One(service_type) => {
            if service_type == "DIDCommMessaging" {
                OneOrList::One("dm".to_string())
            } else {
                OneOrList::One(service_type)
            }
        }
    };
    Ok(ServiceAbbreviatedDidPeer2::new(
        Some(service.id().to_string()),
        service_type,
        service_endpoint,
        routing_keys,
        accept,
    ))
}

#[cfg(test)]
mod tests {
    use did_doc::schema::{
        service::{
            extra_fields::{didcommv2::ExtraFieldsDidCommV2, ServiceKeyKind},
            Service,
        },
        types::uri::Uri,
        verification_method::{VerificationMethod, VerificationMethodType},
    };
    use did_parser::{Did, DidUrl};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{
        peer_did::numalgos::numalgo2::resolve::resolve_numalgo2,
        resolver::options::PublicKeyEncoding,
    };
    use crate::helpers::convert_to_hashmap;

    fn create_verification_method(
        did_full: String,
        key: String,
        verification_type: VerificationMethodType,
    ) -> VerificationMethod {
        VerificationMethod::builder(
            did_full.parse().unwrap(),
            did_full.parse().unwrap(),
            verification_type,
        )
        .add_public_key_multibase(key)
        .build()
    }

    #[test]
    fn test_append_encoded_key_segments() {
        let did = "did:peer:2";
        let key_0 = "z6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc";
        let key_1 = "z6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V";
        let did_full = format!("{}.E{}.V{}", did, key_0, key_1);

        let vm_0 = create_verification_method(
            did_full.to_string(),
            key_0.to_string(),
            VerificationMethodType::X25519KeyAgreementKey2020,
        );
        let vm_1 = create_verification_method(
            did_full.to_string(),
            key_1.to_string(),
            VerificationMethodType::Ed25519VerificationKey2020,
        );

        let did_document = DidDocument::builder(did_full.parse().unwrap())
            .add_key_agreement(vm_0)
            .add_verification_method(vm_1)
            .build();

        let did = append_encoded_key_segments(did.to_string(), &did_document).unwrap();
        assert_eq!(did, did_full);
    }

    #[test]
    fn test_append_encoded_service_segment() {
        let did = "did:peer:2";
        let service = "eyJpZCI6IiNzZXJ2aWNlLTAiLCJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXSwiYSI6WyJkaWRjb21tL3YyIiwiZGlkY29tbS9haXAyO2Vudj1yZmM1ODciXX0";
        let did_expected = format!("{}.S{}", did, service);

        let extra = ExtraFieldsDidCommV2::builder()
            .set_routing_keys(vec![ServiceKeyKind::Reference(
                "did:example:somemediator#somekey".parse().unwrap(),
            )])
            .add_accept("didcomm/aip2;env=rfc587".into())
            .build();

        let service = Service::new(
            Uri::new("#service-0").unwrap(),
            "https://example.com/endpoint".parse().unwrap(),
            OneOrList::One("DIDCommMessaging".to_string()),
            convert_to_hashmap(&extra).unwrap(),
        );

        let did_document = DidDocument::builder(did_expected.parse().unwrap())
            .add_service(service)
            .build();

        let did = append_encoded_service_segment(did.to_string(), &did_document).unwrap();

        let ddo = resolve_numalgo2(&did.parse::<Did>().unwrap(), PublicKeyEncoding::Base58)
            .unwrap()
            .build();

        let ddo_expected = resolve_numalgo2(
            &did_expected.parse::<Did>().unwrap(),
            PublicKeyEncoding::Base58,
        )
        .unwrap()
        .build();

        assert_eq!(ddo, ddo_expected);
        assert_eq!(did, did_expected);
    }

    #[test]
    fn test_append_encoded_segments_error() {
        let did = "did:peer:2";
        let key = "invalid_key";
        let did_full = format!("{}.E{}", did, key);

        let vm = create_verification_method(
            did_full.to_string(),
            key.to_string(),
            VerificationMethodType::X25519KeyAgreementKey2020,
        );

        let did_document = DidDocument::builder(did_full.parse().unwrap())
            .add_key_agreement(vm)
            .build();

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
            did_full.to_string(),
            key_0.to_string(),
            VerificationMethodType::X25519KeyAgreementKey2020,
        );
        let vm_1 = create_verification_method(
            did_full.to_string(),
            key_1.to_string(),
            VerificationMethodType::Ed25519VerificationKey2020,
        );
        let vm_2 = create_verification_method(
            did_full.to_string(),
            key_2.to_string(),
            VerificationMethodType::Ed25519VerificationKey2020,
        );

        let did_document = DidDocument::builder(did_full.parse().unwrap())
            .add_assertion_method(vm_0)
            .add_key_agreement(vm_1)
            .add_verification_method(vm_2)
            .build();

        let did = append_encoded_key_segments(did.to_string(), &did_document).unwrap();
        assert_eq!(did, did_full);
    }

    #[test]
    fn test_append_encoded_key_segments_resolvable_key() {
        let did = "did:peer:2";
        let key = "z6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc";
        let did_full = format!("{}.E{}.V{}", did, key, key);
        let reference = "ref-1";

        let vm = create_verification_method(
            format!("{did_full}#{reference}"),
            key.to_string(),
            VerificationMethodType::X25519KeyAgreementKey2020,
        );

        let did_document = DidDocument::builder(did_full.parse().unwrap())
            .add_verification_method(vm)
            .add_key_agreement_reference(DidUrl::from_fragment(reference.to_string()).unwrap())
            .build();

        let did = append_encoded_key_segments(did.to_string(), &did_document).unwrap();
        assert_eq!(did, did_full);
    }
}
