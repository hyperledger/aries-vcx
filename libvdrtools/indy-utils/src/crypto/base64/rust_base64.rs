use base64::{
    alphabet,
    engine::{general_purpose, DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig},
    Engine,
};
use indy_api_types::errors::prelude::*;

/// Default general purpose configuration, but padding decode mode of 'indifferent' (will decode
/// either)
const ANY_PADDING: GeneralPurposeConfig =
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent);
/// Standard Base64 URL Safe decoding and encoding, with indifference for padding mode when decoding
const URL_SAFE_ANY_PADDING: GeneralPurpose = GeneralPurpose::new(&alphabet::URL_SAFE, ANY_PADDING);

pub fn encode(doc: &[u8]) -> String {
    general_purpose::STANDARD.encode(doc)
}

pub fn decode(doc: &str) -> Result<Vec<u8>, IndyError> {
    general_purpose::STANDARD
        .decode(doc)
        .map_err(|e| e.to_indy(IndyErrorKind::InvalidStructure, "Invalid base64 sequence"))
}

pub fn encode_urlsafe(doc: &[u8]) -> String {
    general_purpose::URL_SAFE.encode(doc)
}

pub fn decode_urlsafe(doc: &str) -> Result<Vec<u8>, IndyError> {
    URL_SAFE_ANY_PADDING.decode(doc).map_err(|e| {
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

    #[test]
    fn decode_urlsafe_works_with_or_without_padding() {
        let result = decode_urlsafe("YWJjZA==");
        assert_eq!(vec![97, 98, 99, 100], result.unwrap());

        let result = decode_urlsafe("YWJjZA");
        assert_eq!(vec![97, 98, 99, 100], result.unwrap());
    }
}
