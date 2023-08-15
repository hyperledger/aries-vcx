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
fn test_deserialization() {}

#[test]
fn test_serialization() {}
