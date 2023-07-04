use indy_api_types::errors::prelude::*;

pub fn encode(doc: &[u8]) -> String {
    base64::encode(doc)
}

pub fn decode(doc: &str) -> Result<Vec<u8>, IndyError> {
    base64::decode(doc)
        .map_err(|e| e.to_indy(IndyErrorKind::InvalidStructure, "Invalid base64 sequence"))
}

pub fn encode_urlsafe(doc: &[u8]) -> String {
    base64::encode_config(doc, base64::URL_SAFE) //TODO switch to URL_SAFE_NO_PAD
}

pub fn decode_urlsafe(doc: &str) -> Result<Vec<u8>, IndyError> {
    base64::decode_config(doc, base64::URL_SAFE_NO_PAD).map_err(|e| {
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

    #[test] // aries-396
    fn encode_base64_urlsafe_and_urlsafe_no_pad_compatible() {
        let data = "Hello World";
        {
            let encoded = base64::encode_config(data, base64::URL_SAFE);
            let decoded_data = base64::decode_config(&encoded, base64::URL_SAFE_NO_PAD).unwrap();
            assert_eq!(data.as_bytes().to_vec(), decoded_data);
        }
        {
            let encoded = base64::encode_config(data, base64::URL_SAFE_NO_PAD);
            let decoded_data = base64::decode_config(&encoded, base64::URL_SAFE).unwrap();
            assert_eq!(data.as_bytes().to_vec(), decoded_data);
        }
    }
}
