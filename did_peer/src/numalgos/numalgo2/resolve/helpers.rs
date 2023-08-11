use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use did_doc::schema::{did_doc::DidDocumentBuilder, service::Service, types::uri::Uri, utils::OneOrList};
use did_doc_sov::extra_fields::{aip1::ExtraFieldsAIP1, didcommv2::ExtraFieldsDidCommV2, ExtraFieldsSov};
use did_parser::Did;
use public_key::Key;

use crate::{
    error::DidPeerError,
    numalgos::numalgo2::{
        purpose::ElementPurpose, service_abbreviated::ServiceAbbreviated,
        verification_method::get_verification_methods_by_key,
    },
    peer_did_resolver::options::PublicKeyEncoding,
};

pub fn process_elements(
    mut did_doc_builder: DidDocumentBuilder<ExtraFieldsSov>,
    did: &Did,
    public_key_encoding: PublicKeyEncoding,
) -> Result<DidDocumentBuilder<ExtraFieldsSov>, DidPeerError> {
    let mut service_index: usize = 0;

    // Skipping one here because the first element is empty string
    for element in did.id()[1..].split('.').skip(1) {
        did_doc_builder = process_element(element, did_doc_builder, &mut service_index, did, public_key_encoding)?;
    }

    Ok(did_doc_builder)
}

fn process_element(
    element: &str,
    mut did_doc_builder: DidDocumentBuilder<ExtraFieldsSov>,
    service_index: &mut usize,
    did: &Did,
    public_key_encoding: PublicKeyEncoding,
) -> Result<DidDocumentBuilder<ExtraFieldsSov>, DidPeerError> {
    let purpose: ElementPurpose = element
        .chars()
        .nth(0)
        .ok_or(DidPeerError::DidValidationError(format!(
            "No purpose code following element separator in '{}'",
            element
        )))?
        .try_into()?;
    let purposeless_element = &element[1..];

    if purpose == ElementPurpose::Service {
        did_doc_builder = process_service_element(&purposeless_element, did_doc_builder, service_index)?;
    } else {
        did_doc_builder =
            process_key_element(&purposeless_element, did_doc_builder, did, public_key_encoding, purpose)?;
    }

    Ok(did_doc_builder)
}

fn process_service_element(
    element: &str,
    mut did_doc_builder: DidDocumentBuilder<ExtraFieldsSov>,
    service_index: &mut usize,
) -> Result<DidDocumentBuilder<ExtraFieldsSov>, DidPeerError> {
    let decoded = STANDARD_NO_PAD.decode(element)?;
    let service: OneOrList<ServiceAbbreviated> = serde_json::from_slice(&decoded)?;

    match service {
        OneOrList::One(service) => {
            did_doc_builder = did_doc_builder.add_service(deabbreviate_service(service, *service_index)?);
            *service_index += 1;
        }
        OneOrList::List(services) => {
            for service in services.into_iter() {
                did_doc_builder = did_doc_builder.add_service(deabbreviate_service(service, *service_index)?);
                *service_index += 1;
            }
        }
    }

    Ok(did_doc_builder)
}

fn process_key_element(
    element: &str,
    mut did_doc_builder: DidDocumentBuilder<ExtraFieldsSov>,
    did: &Did,
    public_key_encoding: PublicKeyEncoding,
    purpose: ElementPurpose,
) -> Result<DidDocumentBuilder<ExtraFieldsSov>, DidPeerError> {
    let key = Key::from_fingerprint(&element)?;
    let vms = get_verification_methods_by_key(&key, did, public_key_encoding)?;

    for vm in vms.into_iter() {
        match purpose {
            ElementPurpose::Assertion => {
                did_doc_builder = did_doc_builder.add_assertion_method(vm);
            }
            ElementPurpose::Encryption => {
                did_doc_builder = did_doc_builder.add_key_agreement(vm);
            }
            ElementPurpose::Verification => {
                did_doc_builder = did_doc_builder.add_verification_method(vm);
            }
            ElementPurpose::CapabilityInvocation => did_doc_builder = did_doc_builder.add_capability_invocation(vm),
            ElementPurpose::CapabilityDelegation => did_doc_builder = did_doc_builder.add_capability_delegation(vm),
            _ => return Err(DidPeerError::UnsupportedPurpose(purpose.into())),
        }
    }

    Ok(did_doc_builder)
}

fn deabbreviate_service(service: ServiceAbbreviated, index: usize) -> Result<Service<ExtraFieldsSov>, DidPeerError> {
    let service_type = match service.service_type() {
        "dm" => "DIDCommMessaging".to_string(),
        t @ _ => t.to_string(),
    };

    let id = format!("#{}-{}", service_type.to_lowercase(), index).parse()?;

    if service.routing_keys().is_empty() {
        build_service_aip1(service, id, service_type)
    } else {
        build_service_didcommv2(service, id, service_type)
    }
}

fn build_service_aip1(
    service: ServiceAbbreviated,
    id: Uri,
    service_type: String,
) -> Result<Service<ExtraFieldsSov>, DidPeerError> {
    Ok(Service::<ExtraFieldsSov>::builder(
        id,
        service.service_endpoint().parse()?,
        ExtraFieldsSov::AIP1(ExtraFieldsAIP1::default()),
    )
    .add_service_type(service_type.to_string())?
    .build())
}

fn build_service_didcommv2(
    service: ServiceAbbreviated,
    id: Uri,
    service_type: String,
) -> Result<Service<ExtraFieldsSov>, DidPeerError> {
    let extra_builder = ExtraFieldsDidCommV2::builder()
        .set_routing_keys(service.routing_keys().to_owned())
        .set_accept(service.accept().to_owned());
    let extra = ExtraFieldsSov::DIDCommV2(extra_builder.build());
    Ok(
        Service::<ExtraFieldsSov>::builder(id, service.service_endpoint().parse()?, extra)
            .add_service_type(service_type.to_string())?
            .build(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use did_doc::schema::utils::OneOrList;
    use did_doc_sov::extra_fields::{AcceptType, ExtraFieldsSov, KeyKind};

    #[test]
    fn test_process_elements_empty_did() {
        let did: Did = "did:peer:2".parse().unwrap();

        let built_ddo = process_elements(
            DidDocumentBuilder::<ExtraFieldsSov>::new(did.clone()),
            &did,
            PublicKeyEncoding::Base58,
        )
        .unwrap()
        .build();
        assert_eq!(built_ddo.id().to_string(), did.to_string());
    }

    #[test]
    fn test_process_elements_with_multiple_elements() {
        let did: Did = "did:peer:2\
                        .Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V\
                        .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCJ9"
            .parse()
            .unwrap();

        let processed_did_doc_builder = process_elements(
            DidDocumentBuilder::<ExtraFieldsSov>::new(did.clone()),
            &did,
            PublicKeyEncoding::Multibase,
        )
        .unwrap();
        let built_ddo = processed_did_doc_builder.build();

        assert_eq!(built_ddo.id().to_string(), did.to_string());
        assert_eq!(built_ddo.verification_method().len(), 1);
        assert_eq!(built_ddo.service().len(), 1);
    }

    #[test]
    fn test_process_elements_error_on_invalid_element() {
        let did: Did = "did:peer:2\
                        .Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V\
                        .SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCJ9\
                        .Xinvalid"
            .parse()
            .unwrap();

        match process_elements(
            DidDocumentBuilder::<ExtraFieldsSov>::new(did.clone()),
            &did,
            PublicKeyEncoding::Multibase,
        ) {
            Ok(_) => panic!("Expected Err, got Ok"),
            Err(e) => {
                assert!(matches!(e, DidPeerError::UnsupportedPurpose('X')));
            }
        }
    }

    #[test]
    fn test_process_service_element_one_service() {
        let purposeless_service_element = "eyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCJ9";
        let did: Did = format!("did:peer:2.S{}", purposeless_service_element).parse().unwrap();
        let mut index = 0;
        let ddo_builder = DidDocumentBuilder::<ExtraFieldsSov>::new(did.clone());
        let built_ddo = process_service_element(purposeless_service_element, ddo_builder, &mut index)
            .unwrap()
            .build();
        assert_eq!(built_ddo.service().len(), 1);
        let service = built_ddo.service().first().unwrap();
        assert_eq!(service.id().to_string(), "#didcommmessaging-0".to_string());
        assert_eq!(service.service_type().to_string(), "DIDCommMessaging".to_string());
        assert_eq!(
            service.service_endpoint().to_string(),
            "https://example.com/endpoint".to_string()
        );
    }

    #[test]
    fn test_process_service_element_multiple_services() {
        let purposeless_service_element = "W3sidCI6ImRtIiwicyI6Imh0dHBzOi8vZXhhbXBsZS5jb20vZW5kcG9pbnQiLCJyIjpbImRpZDpleGFtcGxlOnNvbWVtZWRpYXRvciNzb21la2V5Il19LHsidCI6ImV4YW1wbGUiLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludDIiLCJyIjpbImRpZDpleGFtcGxlOnNvbWVtZWRpYXRvciNzb21la2V5MiJdLCJhIjpbImRpZGNvbW0vdjIiLCJkaWRjb21tL2FpcDI7ZW52PXJmYzU4NyJdfV0";
        let did: Did = format!("did:peer:2.S{}", purposeless_service_element).parse().unwrap();
        let mut index = 0;
        let ddo_builder = DidDocumentBuilder::<ExtraFieldsSov>::new(did.clone());
        let built_ddo = process_service_element(purposeless_service_element, ddo_builder, &mut index)
            .unwrap()
            .build();

        assert_eq!(built_ddo.service().len(), 2);

        let first_service = built_ddo.service().first().unwrap();
        assert_eq!(first_service.id().to_string(), "#didcommmessaging-0".to_string());
        assert_eq!(first_service.service_type().to_string(), "DIDCommMessaging".to_string());
        assert_eq!(
            first_service.extra().first_routing_key().unwrap().to_string(),
            "did:example:somemediator#somekey".to_string()
        );
        assert_eq!(
            first_service.service_endpoint().to_string(),
            "https://example.com/endpoint".to_string()
        );

        let second_service = built_ddo.service().get(1).unwrap();
        assert_eq!(second_service.id().to_string(), "#example-1".to_string());
        assert_eq!(second_service.service_type().to_string(), "example".to_string());
        assert_eq!(
            second_service.service_endpoint().to_string(),
            "https://example.com/endpoint2".to_string()
        );
        assert_eq!(
            second_service.extra().accept().unwrap(),
            vec![
                AcceptType::DIDCommV2,
                AcceptType::Other("didcomm/aip2;env=rfc587".to_string())
            ]
        );
        assert_eq!(
            second_service.extra().first_routing_key().unwrap().to_string(),
            "did:example:somemediator#somekey2".to_string()
        );
    }

    #[test]
    fn test_process_key_element() {
        let purposeless_key_element = "z6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V";
        let did: Did = format!("did:peer:2.V{}", purposeless_key_element).parse().unwrap();

        let ddo_builder = DidDocumentBuilder::<ExtraFieldsSov>::new(did.clone());
        let public_key_encoding = PublicKeyEncoding::Multibase;
        let built_ddo = process_key_element(
            purposeless_key_element,
            ddo_builder,
            &did,
            public_key_encoding,
            ElementPurpose::Verification,
        )
        .unwrap()
        .build();

        assert_eq!(built_ddo.verification_method().len(), 1);
        let vm = built_ddo.verification_method().first().unwrap();
        assert_eq!(vm.id().to_string(), "#6MkqRYqQ");
        assert_eq!(vm.controller().to_string(), did.to_string());
    }

    #[test]
    fn test_process_key_element_negative() {
        let did: Did = "did:peer:2".parse().unwrap();
        assert!(process_key_element(
            "z6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V",
            DidDocumentBuilder::<ExtraFieldsSov>::new(did.clone()),
            &did,
            PublicKeyEncoding::Multibase,
            ElementPurpose::Service
        )
        .is_err());
    }

    #[test]
    fn test_deabbreviate_service_aip1() {
        let service_abbreviated = ServiceAbbreviated::from_parts("dm", "https://example.com/endpoint", &[], &[]);
        let index = 0;

        let service = deabbreviate_service(service_abbreviated, index).unwrap();

        assert_eq!(
            service.service_type().clone(),
            OneOrList::One("DIDCommMessaging".to_string())
        );
        assert_eq!(service.id().to_string(), "#didcommmessaging-0");

        assert!(matches!(service.extra(), ExtraFieldsSov::AIP1(_)));
    }

    #[test]
    fn test_deabbreviate_service_didcommv2() {
        let routing_keys = vec![KeyKind::Value("key1".to_string())];
        let service_abbreviated =
            ServiceAbbreviated::from_parts("dm", "https://example.com/endpoint", &routing_keys, &[]);
        let index = 0;

        let service = deabbreviate_service(service_abbreviated, index).unwrap();

        assert_eq!(
            service.service_type().clone(),
            OneOrList::One("DIDCommMessaging".to_string())
        );
        assert_eq!(service.id().to_string(), "#didcommmessaging-0");

        match service.extra() {
            ExtraFieldsSov::DIDCommV2(extra) => {
                assert_eq!(extra.routing_keys(), &routing_keys);
            }
            _ => panic!("Expected ExtraFieldsSov::DIDCommV2"),
        }
    }

    #[test]
    fn test_build_service_aip1() {
        let routing_keys = vec![KeyKind::Value("key1".to_string())];
        let service_abbreviated = ServiceAbbreviated::from_parts(
            "dm",
            "https://example.com/endpoint",
            routing_keys.as_ref(),
            vec![].as_ref(),
        );

        let id = Uri::new("did:peer:2").unwrap();
        let service_type = "DIDCommMessaging".to_string();

        let service = build_service_aip1(service_abbreviated, id, service_type).unwrap();

        assert_eq!(service.id().to_string(), "did:peer:2");
        assert_eq!(
            service.service_type().clone(),
            OneOrList::One("DIDCommMessaging".to_string())
        );

        match service.extra() {
            ExtraFieldsSov::AIP1(_) => { /* This is expected */ }
            _ => panic!("Expected ExtraFieldsSov::AIP1"),
        }
    }

    #[test]
    fn test_build_service_didcommv2() {
        let routing_keys = vec![KeyKind::Value("key1".to_string())];
        let accept = vec![AcceptType::DIDCommV2];
        let service_abbreviated = ServiceAbbreviated::from_parts(
            "dm",
            "https://example.com/endpoint",
            routing_keys.as_ref(),
            accept.as_ref(),
        );

        let id = Uri::new("did:peer:2").unwrap();
        let service_type = "DIDCommMessaging".to_string();

        let service = build_service_didcommv2(service_abbreviated, id, service_type).unwrap();

        assert_eq!(service.id().to_string(), "did:peer:2");
        assert_eq!(
            service.service_type().clone(),
            OneOrList::One("DIDCommMessaging".to_string())
        );

        match service.extra() {
            ExtraFieldsSov::DIDCommV2(extra) => {
                assert_eq!(extra.routing_keys(), &routing_keys);
                assert_eq!(extra.accept(), &accept);
            }
            _ => panic!("Expected ExtraFieldsSov::DIDCommV2"),
        }
    }
}
