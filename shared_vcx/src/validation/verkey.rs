use bs58;

use crate::errors::error::{SharedVcxError, SharedVcxErrorKind, SharedVcxResult};

pub fn validate_verkey(verkey: &str) -> SharedVcxResult<String> {
    let check_verkey = String::from(verkey);
    match bs58::decode(check_verkey.clone()).into_vec() {
        Ok(ref x) if x.len() == 32 => Ok(check_verkey),
        Ok(x) => Err(SharedVcxError::from_msg(
            SharedVcxErrorKind::InvalidVerkey,
            format!("Invalid verkey length, expected 32 bytes, decoded {} bytes", x.len()),
        )),
        Err(err) => Err(SharedVcxError::from_msg(
            SharedVcxErrorKind::NotBase58,
            format!("Verkey is not valid base58, details: {}", err),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_verkey_is_b58_and_valid_length() {
        let verkey = "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A";
        match validate_verkey(&verkey) {
            Err(_) => panic!("Should be valid verkey"),
            Ok(x) => assert_eq!(x, verkey),
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_verkey_is_b58_but_invalid_length() {
        let verkey = "8XFh8yBzrpJQmNyZzgoT";
        match validate_verkey(&verkey) {
            Err(x) => assert_eq!(x.kind(), SharedVcxErrorKind::InvalidVerkey),
            Ok(_) => panic!("Should be invalid verkey"),
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_validate_verkey_with_non_base58() {
        let verkey = "*kVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A";
        match validate_verkey(&verkey) {
            Err(x) => assert_eq!(x.kind(), SharedVcxErrorKind::NotBase58),
            Ok(_) => panic!("Should be invalid verkey"),
        }
    }
}
