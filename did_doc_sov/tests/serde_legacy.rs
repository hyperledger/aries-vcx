use did_doc_sov::{
    extra_fields::{AcceptType, KeyKind},
    legacy::wrapper::LegacyOrNew,
    service::ServiceType,
    DidDocumentSov,
};

const LEGACY_DID_DOC_JSON: &str = r##"
{
  "@context": "https://w3id.org/did/v1",
  "id": "testid",
  "publicKey": [
    {
      "id": "testid#1",
      "type": "Ed25519VerificationKey2018",
      "controller": "testid",
      "publicKeyBase58": "FTf8juor9EQSwL4RDHyVdSVuJtLSXzmVZX5fBAmMyH6V"
    }
  ],
  "authentication": [
    {
      "type": "Ed25519SignatureAuthentication2018",
      "publicKey": "testid#1"
    }
  ],
  "service": [
    {
      "id": "did:example:123456789abcdefghi;indy",
      "type": "IndyAgent",
      "priority": 0,
      "recipientKeys": [
        "FTf8juor9EQSwL4RDHyVdSVuJtLSXzmVZX5fBAmMyH6V"
      ],
      "routingKeys": [],
      "serviceEndpoint": "https://service-endpoint.org"
    }
  ]
}
"##;

#[test]
fn test_deserialization_legacy() {
    let did_doc: DidDocumentSov = serde_json::from_str(LEGACY_DID_DOC_JSON).unwrap();
    println!("{:#?}", did_doc);
}

#[test]
fn test_serialization_legacy() {}
