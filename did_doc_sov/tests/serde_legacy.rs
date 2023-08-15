use did_doc::schema::verification_method::VerificationMethodKind;
use did_doc_sov::{extra_fields::KeyKind, DidDocumentSov};

const LEGACY_DID_DOC_JSON: &str = r##"
{
   "@context": "https://w3id.org/did/v1",
   "id": "2ZHFFhzA2XtTD6hJqzL7ux",
   "publicKey": [
       {
           "id": "1",
           "type": "Ed25519VerificationKey2018",
           "controller": "2ZHFFhzA2XtTD6hJqzL7ux",
           "publicKeyBase58": "rCw3x5h1jS6gPo7rRrt3EYbXXe5nNjnGbdf1jAwUxuj"
       }
   ],
   "authentication": [
       {
           "type": "Ed25519SignatureAuthentication2018",
           "publicKey": "2ZHFFhzA2XtTD6hJqzL7ux#1"
       }
   ],
   "service": [
       {
           "id": "did:example:123456789abcdefghi;indy",
           "type": "IndyAgent",
           "priority": 0,
           "recipientKeys": [
               "2ZHFFhzA2XtTD6hJqzL7ux#1"
           ],
           "routingKeys": [
               "8Ps2WosJ9AV1eXPoJKsEJdM3NchPhSyS8qFt6LQUTKv2",
               "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR"
           ],
           "serviceEndpoint": "http://localhost:8080/agency/msg"
       }
   ]
}
"##;

#[test]
fn test_deserialization_legacy() {
    let did_doc: DidDocumentSov = serde_json::from_str(LEGACY_DID_DOC_JSON).unwrap();
    println!("{:#?}", did_doc);
    assert_eq!(did_doc.id().to_string(), "2ZHFFhzA2XtTD6hJqzL7ux");
    assert_eq!(did_doc.verification_method().len(), 1);
    assert_eq!(did_doc.authentication().len(), 0);
    assert_eq!(did_doc.assertion_method().len(), 0);
    assert_eq!(did_doc.key_agreement().len(), 1);
    assert_eq!(did_doc.service().len(), 1);

    let authentication = did_doc.verification_method().first().unwrap();
    // assert_eq!(authentication.id().to_string(), "2ZHFFhzA2XtTD6hJqzL7ux");
    assert_eq!(
        authentication.public_key().base58().unwrap(),
        "rCw3x5h1jS6gPo7rRrt3EYbXXe5nNjnGbdf1jAwUxuj"
    );

    let key_agreement = match did_doc.key_agreement().first().unwrap() {
        VerificationMethodKind::Resolved(res) => res,
        VerificationMethodKind::Resolvable(_) => panic!("Expected resolved verification method"),
    };
    // assert_eq!(key_agreement.id().to_string(), "2ZHFFhzA2XtTD6hJqzL7ux");
    assert_eq!(
        key_agreement.public_key().base58().unwrap(),
        "rCw3x5h1jS6gPo7rRrt3EYbXXe5nNjnGbdf1jAwUxuj"
    );

    let service = did_doc.service().first().unwrap();
    assert_eq!(service.id().to_string(), "did:example:123456789abcdefghi;indy");
    assert_eq!(
        service.service_endpoint().to_string().as_str(),
        "http://localhost:8080/agency/msg"
    );

    let recipient_key = match service.extra().first_recipient_key().unwrap() {
        KeyKind::Reference(did_url) => did_doc.dereference_key(did_url).unwrap().public_key().base58().unwrap(),
        KeyKind::Value(value) => value.clone(),
        KeyKind::DidKey(_) => panic!("Expected reference or value"),
    };
    assert_eq!(recipient_key, "rCw3x5h1jS6gPo7rRrt3EYbXXe5nNjnGbdf1jAwUxuj");
    assert_eq!(service.extra().routing_keys().unwrap().len(), 2);
}

#[test]
fn test_serialization_legacy() {}
