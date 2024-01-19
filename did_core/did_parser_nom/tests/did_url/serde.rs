use did_parser_nom::DidUrl;
use serde_test::{assert_tokens, Token};

const DID_URL: &str = "did:example:namespace:123456789abcdefghi/path?query=value#fragment";

#[test]
fn test_did_url_serde() {
    assert_tokens(
        &DidUrl::parse(DID_URL.to_string()).unwrap(),
        &[Token::Str(DID_URL)],
    );
}
