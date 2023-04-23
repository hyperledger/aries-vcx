use std::str::FromStr;

use did_doc_builder::schema::{
    did_doc::DIDDocument,
    types::{jsonwebkey::JsonWebKey, uri::Uri},
    verification_method::{VerificationMethodAlias, VerificationMethodBuilder},
};
use did_parser::{ParsedDID, ParsedDIDUrl};
use serde_json::Value;

const VALID_DID_DOC_JSON: &str = r##"
{
  "@context": [
    "https://w3.org/ns/did/v1",
    "https://w3id.org/security/suites/ed25519-2018/v1"
  ],
  "id": "did:web:did.actor:alice",
  "alsoKnownAs": [
      "https://example.com/user-profile/123"
  ],
  "publicKey": [
    {
      "id": "did:web:did.actor:alice#z6MkrmNwty5ajKtFqc1U48oL2MMLjWjartwc5sf2AihZwXDN",
      "controller": "did:web:did.actor:alice",
      "type": "Ed25519VerificationKey2018",
      "publicKeyBase58": "DK7uJiq9PnPnj7AmNZqVBFoLuwTjT1hFPrk6LSjZ2JRz"
    }
  ],
  "authentication": [
    "did:web:did.actor:alice#z6MkrmNwty5ajKtFqc1U48oL2MMLjWjartwc5sf2AihZwXDN"
  ],
  "assertionMethod": [
    "did:web:did.actor:alice#z6MkrmNwty5ajKtFqc1U48oL2MMLjWjartwc5sf2AihZwXDN"
  ],
  "capabilityDelegation": [
    "did:web:did.actor:alice#z6MkrmNwty5ajKtFqc1U48oL2MMLjWjartwc5sf2AihZwXDN"
  ],
  "capabilityInvocation": [
    "did:web:did.actor:alice#z6MkrmNwty5ajKtFqc1U48oL2MMLjWjartwc5sf2AihZwXDN"
  ],
  "verificationMethod": [
    {
      "id": "#g1",
      "controller": "did:web:did.actor:alice",
      "type": "JsonWebKey2020",
      "publicKeyJwk": {
        "kty": "EC",
        "crv": "BLS12381_G1",
        "x": "hxF12gtsn9ju4-kJq2-nUjZQKVVWpcBAYX5VHnUZMDilClZsGuOaDjlXS8pFE1GG"
      }
    },
    {
      "id": "#g2",
      "controller": "did:web:did.actor:alice",
      "type": "JsonWebKey2020",
      "publicKeyJwk": {
        "kty": "EC",
        "crv": "BLS12381_G2",
        "x": "l4MeBsn_OGa2OEDtHeHdq0TBC8sYh6QwoI7QsNtZk9oAru1OnGClaAPlMbvvs73EABDB6GjjzybbOHarkBmP6pon8H1VuMna0nkEYihZi8OodgdbwReDiDvWzZuXXMl-"
      }
    }
  ],
  "keyAgreement": [
    {
      "id": "did:web:did.actor:alice#zC8GybikEfyNaausDA4mkT4egP7SNLx2T1d1kujLQbcP6h",
      "type": "X25519KeyAgreementKey2019",
      "controller": "did:web:did.actor:alice",
      "publicKeyBase58": "CaSHXEvLKS6SfN9aBfkVGBpp15jSnaHazqHgLHp8KZ3Y"
    }
  ]
}
"##;

#[test]
fn test_deserialization() {
    let did_doc: DIDDocument = serde_json::from_str(VALID_DID_DOC_JSON).unwrap();

    assert_eq!(
        did_doc.id(),
        &ParsedDID::from_str("did:web:did.actor:alice").unwrap()
    );
    assert_eq!(
        did_doc.also_known_as(),
        vec![Uri::from_str("https://example.com/user-profile/123").unwrap()]
    );

    let controller = ParsedDID::from_str("did:web:did.actor:alice").unwrap();

    let pk_id = ParsedDIDUrl::from_str(
        "did:web:did.actor:alice#z6MkrmNwty5ajKtFqc1U48oL2MMLjWjartwc5sf2AihZwXDN",
    )
    .unwrap();

    let vm1_id = ParsedDIDUrl::from_str("#g1").unwrap();
    let vm1 = VerificationMethodBuilder::new(
        vm1_id.clone(),
        controller.clone(),
        "JsonWebKey2020".to_string(),
    )
    .add_public_key_jwk(
        JsonWebKey::from_str(
            r#"{
                "kty": "EC",
                "crv": "BLS12381_G1",
                "x": "hxF12gtsn9ju4-kJq2-nUjZQKVVWpcBAYX5VHnUZMDilClZsGuOaDjlXS8pFE1GG"
            }"#,
        )
        .unwrap(),
    )
    .build()
    .unwrap();

    let vm2_id = ParsedDIDUrl::from_str("#g2").unwrap();
    let vm2 = VerificationMethodBuilder::new(
        vm2_id.clone(),
        controller.clone(),
        "JsonWebKey2020".to_string(),
    )
    .add_public_key_jwk(
        JsonWebKey::from_str(
            r#"{
                "kty": "EC",
                "crv": "BLS12381_G2",
                "x": "l4MeBsn_OGa2OEDtHeHdq0TBC8sYh6QwoI7QsNtZk9oAru1OnGClaAPlMbvvs73EABDB6GjjzybbOHarkBmP6pon8H1VuMna0nkEYihZi8OodgdbwReDiDvWzZuXXMl-"
            }"#,
        )
        .unwrap(),
    )
    .build().unwrap();

    assert_eq!(did_doc.verification_method().get(0).unwrap().clone(), vm1);
    assert_eq!(did_doc.verification_method().get(1).unwrap().clone(), vm2);

    assert_eq!(
        did_doc.authentication(),
        &[VerificationMethodAlias::VerificationMethodReference(
            pk_id.clone()
        )]
    );

    assert_eq!(
        did_doc.assertion_method(),
        &[VerificationMethodAlias::VerificationMethodReference(
            pk_id.clone()
        )]
    );

    assert_eq!(
        did_doc.capability_delegation(),
        &[VerificationMethodAlias::VerificationMethodReference(
            pk_id.clone()
        )]
    );

    assert_eq!(
        did_doc.capability_invocation(),
        &[VerificationMethodAlias::VerificationMethodReference(
            pk_id.clone()
        )]
    );

    assert_eq!(
        did_doc.extra("publicKey").unwrap().clone(),
        Value::Array(vec![Value::Object(
            serde_json::from_str(
                r#"{
                    "id": "did:web:did.actor:alice#z6MkrmNwty5ajKtFqc1U48oL2MMLjWjartwc5sf2AihZwXDN",
                    "type": "Ed25519VerificationKey2018",
                    "controller": "did:web:did.actor:alice",
                    "publicKeyBase58": "DK7uJiq9PnPnj7AmNZqVBFoLuwTjT1hFPrk6LSjZ2JRz"
                }"#
            )
            .unwrap()
        )])
    );

    let ka1_id = ParsedDIDUrl::from_str(
        "did:web:did.actor:alice#zC8GybikEfyNaausDA4mkT4egP7SNLx2T1d1kujLQbcP6h",
    )
    .unwrap();
    let ka1 = VerificationMethodBuilder::new(
        ka1_id.clone(),
        controller.clone(),
        "X25519KeyAgreementKey2019".to_string(),
    )
    .add_extra(
        "publicKeyBase58".to_string(),
        Value::String("CaSHXEvLKS6SfN9aBfkVGBpp15jSnaHazqHgLHp8KZ3Y".to_string()),
    )
    .build()
    .unwrap();

    assert_eq!(
        did_doc.key_agreement(),
        &[VerificationMethodAlias::VerificationMethod(ka1)]
    );
}

#[test]
fn test_serialization() {
    let did_doc: DIDDocument = serde_json::from_str(VALID_DID_DOC_JSON).unwrap();

    let serialized_json = serde_json::to_string(&did_doc).unwrap();

    let original_json_value: DIDDocument = serde_json::from_str(VALID_DID_DOC_JSON).unwrap();
    let serialized_json_value: DIDDocument = serde_json::from_str(&serialized_json).unwrap();
    assert_eq!(serialized_json_value, original_json_value);
}
