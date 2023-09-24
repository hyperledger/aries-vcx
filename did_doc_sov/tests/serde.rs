use did_doc_sov::{
    extra_fields::{AcceptType, KeyKind},
    service::ServiceType,
    DidDocumentSov,
};

const DID_DOC_DATA: &str = r#"
{
    "@context": [
        "https://www.w3.org/ns/did/v1",
        "https://w3id.org/security/suites/ed25519-2018/v1",
        "https://w3id.org/security/suites/x25519-2019/v1"
    ],
    "id": "did:sov:HR6vs6GEZ8rHaVgjg2WodM",
    "verificationMethod": [
        {
            "type": "Ed25519VerificationKey2018",
            "id": "did:sov:HR6vs6GEZ8rHaVgjg2WodM#key-1",
            "controller": "did:sov:HR6vs6GEZ8rHaVgjg2WodM",
            "publicKeyBase58": "9wvq2i4xUa5umXoThe83CDgx1e5bsjZKJL4DEWvTP9qe"
        },
        {
            "type": "X25519KeyAgreementKey2019",
            "id": "did:sov:HR6vs6GEZ8rHaVgjg2WodM#key-agreement-1",
            "controller": "did:sov:HR6vs6GEZ8rHaVgjg2WodM",
            "publicKeyBase58": "3mHtKcQFEzqeUcnce5BAuzAgLEbqKaV542pUf9xQ5Pf8"
        }
    ],
    "authentication": [
        "did:sov:HR6vs6GEZ8rHaVgjg2WodM#key-1"
    ],
    "assertionMethod": [
        "did:sov:HR6vs6GEZ8rHaVgjg2WodM#key-1"
    ],
    "keyAgreement": [
        "did:sov:HR6vs6GEZ8rHaVgjg2WodM#key-agreement-1"
    ],
    "service": [
        {
            "id": "did:sov:HR6vs6GEZ8rHaVgjg2WodM#endpoint",
            "type": "endpoint",
            "serviceEndpoint": "https://example.com/endpoint"
        },
        {
            "id": "did:sov:HR6vs6GEZ8rHaVgjg2WodM#did-communication",
            "type": "did-communication",
            "priority": 0,
            "recipientKeys": [
                "did:sov:HR6vs6GEZ8rHaVgjg2WodM#key-agreement-1"
            ],
            "routingKeys": [],
            "accept": [
                "didcomm/aip2;env=rfc19"
            ],
            "serviceEndpoint": "https://example.com/endpoint"
        },
        {
          "id": "did:sov:HR6vs6GEZ8rHaVgjg2WodM#didcomm-1",
          "type": "DIDComm",
          "accept": [
            "didcomm/v2"
          ],
          "routingKeys": [],
          "serviceEndpoint": "https://example.com/endpoint"
        }
    ]
}
"#;

#[test]
fn test_deserialization() {
    let did_doc = serde_json::from_str::<DidDocumentSov>(DID_DOC_DATA).unwrap();
    assert_eq!(did_doc.id().to_string(), "did:sov:HR6vs6GEZ8rHaVgjg2WodM");
    assert_eq!(did_doc.verification_method().len(), 2);
    assert_eq!(did_doc.authentication().len(), 1);
    assert_eq!(did_doc.assertion_method().len(), 1);
    assert_eq!(did_doc.key_agreement().len(), 1);
    assert_eq!(did_doc.service().len(), 3);

    let services = did_doc.service();
    let first_service = services.get(0).unwrap();
    assert_eq!(
        first_service.service_endpoint().to_string(),
        "https://example.com/endpoint"
    );
    assert_eq!(first_service.service_type(), ServiceType::AIP1);

    let second_service = services.get(1).unwrap();
    assert_eq!(
        second_service.id().to_string(),
        "did:sov:HR6vs6GEZ8rHaVgjg2WodM#did-communication"
    );
    assert_eq!(second_service.service_type(), ServiceType::DIDCommV1);
    assert_eq!(
        second_service.service_endpoint().to_string(),
        "https://example.com/endpoint"
    );

    let third_service = services.get(2).unwrap();
    assert_eq!(
        third_service.id().to_string(),
        "did:sov:HR6vs6GEZ8rHaVgjg2WodM#didcomm-1"
    );
    assert_eq!(third_service.service_type(), ServiceType::DIDCommV2);
    assert_eq!(
        third_service.service_endpoint().to_string(),
        "https://example.com/endpoint"
    );

    let second_extra = second_service.extra();
    assert!(!second_extra.recipient_keys().unwrap().is_empty());
    assert_eq!(second_extra.routing_keys().unwrap().len(), 0);
    assert!(second_extra.first_recipient_key().is_ok());
    assert!(second_extra.first_routing_key().is_err());
    assert_eq!(
        second_extra.accept().unwrap().get(0).unwrap().clone(),
        AcceptType::DIDCommV1
    );
    assert_eq!(second_extra.priority().unwrap(), 0);

    let third_extra = third_service.extra();
    assert!(third_extra.recipient_keys().is_err());
    assert_eq!(third_extra.routing_keys().unwrap().len(), 0);
    assert!(third_extra.first_recipient_key().is_err());
    assert!(third_extra.first_routing_key().is_err());
    assert_eq!(
        third_extra.accept().unwrap().get(0).unwrap().clone(),
        AcceptType::DIDCommV2
    );
    assert!(third_extra.priority().is_err());

    if let KeyKind::Reference(reference) = second_extra.first_recipient_key().unwrap() {
        let vm = did_doc.dereference_key(reference).unwrap();
        assert_eq!(
            vm.id().to_string(),
            "did:sov:HR6vs6GEZ8rHaVgjg2WodM#key-agreement-1"
        );
    } else {
        panic!("Expected reference key kind");
    }
}

#[test]
fn test_deserialization_and_serialization() {
    let did_doc_1 = serde_json::from_str::<DidDocumentSov>(DID_DOC_DATA).unwrap();
    let serialized = serde_json::to_string_pretty(&did_doc_1).unwrap();
    let did_doc_2 = serde_json::from_str::<DidDocumentSov>(&serialized).unwrap();
    assert_eq!(did_doc_1, did_doc_2);
}
