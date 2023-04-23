use crate::ParsedDIDUrl;

pub fn is_valid_did(did: &str) -> bool {
    match ParsedDIDUrl::parse(did.to_string()) {
        Ok(parsed_did) => {
            parsed_did.path().is_none()
                && parsed_did.fragment().is_none()
                && parsed_did.queries().is_empty()
                && parsed_did.params().is_empty()
        }
        Err(_) => false,
    }
}

pub fn is_valid_did_url(did_url: &str) -> bool {
    ParsedDIDUrl::parse(did_url.to_string()).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_did() {
        let valid_did = "did:example:123456789abcdefghi";
        let invalid_did1 = "not-a-did:123456789abcdefghi";
        let invalid_did2 = "did:example:123456789abcdefghi/path";
        let invalid_did3 = "did:example:123456789abcdefghi?query=value";
        let invalid_did4 = "did:example:123456789abcdefghi#fragment";

        assert!(is_valid_did(valid_did));
        assert!(!is_valid_did(invalid_did1));
        assert!(!is_valid_did(invalid_did2));
        assert!(!is_valid_did(invalid_did3));
        assert!(!is_valid_did(invalid_did4));
    }

    #[test]
    fn test_is_valid_did_url() {
        let valid_did_url1 = "did:example:123456789abcdefghi";
        let valid_did_url2 = "did:example:123456789abcdefghi/path";
        let valid_did_url3 = "did:example:123456789abcdefghi?query=value";
        let valid_did_url4 = "did:example:123456789abcdefghi#fragment";
        let valid_did_url5 = "did:example:123456789abcdefghi/path?query=value#fragment";
        let invalid_did_url = "not-a-did:123456789abcdefghi";

        assert!(is_valid_did_url(valid_did_url1));
        assert!(is_valid_did_url(valid_did_url2));
        assert!(is_valid_did_url(valid_did_url3));
        assert!(is_valid_did_url(valid_did_url4));
        assert!(is_valid_did_url(valid_did_url5));
        assert!(!is_valid_did_url(invalid_did_url));
    }
}
