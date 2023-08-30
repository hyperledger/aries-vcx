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

const VERKEY_BASE58: &str = "6MkfJTyeCL8MGvZntdpXzpitL6bM6uwCFz8xcYar18xQBh7";

const DID_PEER: &str = "did:peer:2.Vz6MkfJTyeCL8MGvZntdpXzpitL6bM6uwCFz8xcYar18xQBh7.SeyJwcmlvcml0eSI6MCwiciI6WyI4UHMyV29zSjlBVjFlWFBvSktzRUpkTTNOY2hQaFN5UzhxRnQ2TFFVVEt2MiIsIkhlemNlMlVXTVozd1VoVmtoMkxmS1NzOG5Eeld3enMyV2luN0V6Tk4zWWFSIl0sInJlY2lwaWVudEtleXMiOlsiMlpIRkZoekEyWHRURDZoSnF6TDd1eCMxIl0sInMiOiJodHRwOi8vbG9jYWxob3N0OjgwODAvYWdlbmN5L21zZyIsInQiOiJJbmR5QWdlbnQifQ";

#[test]
fn test_deserialization_legacy() {
    let did_doc: DidDocumentSov = serde_json::from_str(LEGACY_DID_DOC_JSON).unwrap();
    println!("{:#?}", serde_json::to_string_pretty(&did_doc).unwrap());
    assert_eq!(did_doc.id().to_string(), DID_PEER);
    assert_eq!(did_doc.verification_method().len(), 1);
    assert_eq!(did_doc.authentication().len(), 0);
    assert_eq!(did_doc.assertion_method().len(), 0);
    assert_eq!(did_doc.key_agreement().len(), 0);
    assert_eq!(did_doc.service().len(), 1);

    let verification_method = did_doc.verification_method().first().unwrap();
    assert_eq!(verification_method.id().to_string(), "#1");
    assert_eq!(verification_method.controller().to_string(), DID_PEER);
    assert_eq!(
        verification_method.public_key().unwrap().prefixless_fingerprint(),
        VERKEY_BASE58
    );

    let service = did_doc.service().first().unwrap();
    assert_eq!(service.id().to_string(), "did:example:123456789abcdefghi;indy");
    assert_eq!(
        service.service_endpoint().to_string().as_str(),
        "http://localhost:8080/agency/msg"
    );

    let recipient_key = match service.extra().first_recipient_key().unwrap() {
        KeyKind::Reference(did_url) => did_doc
            .dereference_key(did_url)
            .unwrap()
            .public_key()
            .unwrap()
            .prefixless_fingerprint(),
        _ => panic!("Expected reference"),
    };
    assert_eq!(recipient_key, VERKEY_BASE58);
    assert_eq!(service.extra().priority().unwrap(), 0);
    assert_eq!(service.extra().routing_keys().unwrap().len(), 2);
}
