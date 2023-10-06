use base64::{engine::general_purpose, Engine};
use indy_api_types::errors::prelude::*;

pub fn encode(doc: &[u8]) -> String {
    general_purpose::STANDARD.encode(doc)
}

pub fn decode(doc: &str) -> Result<Vec<u8>, IndyError> {
    general_purpose::STANDARD
        .decode(doc)
        .map_err(|e| e.to_indy(IndyErrorKind::InvalidStructure, "Invalid base64 sequence"))
}

pub fn encode_urlsafe(doc: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(doc)
}

pub fn decode_urlsafe(doc: &str) -> Result<Vec<u8>, IndyError> {
    general_purpose::URL_SAFE_NO_PAD.decode(doc).map_err(|e| {
        e.to_indy(
            IndyErrorKind::InvalidStructure,
            "Invalid base64URL_SAFE sequence",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_works() {
        let result = encode(&[1, 2, 3]);
        assert_eq!("AQID", &result);
    }

    #[test]
    fn decode_works() {
        let result = decode("AQID");

        assert!(result.is_ok(), "Got error");
        assert_eq!(&[1, 2, 3], &result.unwrap()[..]);
    }

    #[test]
    fn encode_urlsafe_works() {
        let result = encode_urlsafe(&[1, 2, 3]);
        assert_eq!("AQID", &result);
    }

    #[test]
    fn decode_urlsafe_works() {
        let result = decode_urlsafe("AQID");

        assert!(result.is_ok(), "Got error");
        assert_eq!(&[1, 2, 3], &result.unwrap()[..]);
    }
}
