use did_parser_nom::Did;
use serde_test::{assert_tokens, Token};

const DID: &str = "did:example:123456789abcdefghi";

#[test]
fn test_did_serialization() {
    assert_tokens(&Did::parse(DID.to_string()).unwrap(), &[Token::Str(DID)]);
}

#[test]
fn test_did_deserialization() {
    assert_tokens(&Did::parse(DID.to_string()).unwrap(), &[Token::Str(DID)]);
}
