use regex::Regex;

use bs58;
use crate::errors::error::{ErrorAgencyClient, ErrorKindAgencyClient, AgencyClientResult};

lazy_static! {
    pub static ref REGEX: Regex = Regex::new("did:([a-z0-9]+):([a-zA-Z0-9:.-_]*)").unwrap();
}

pub fn is_fully_qualified(entity: &str) -> bool {
    REGEX.is_match(entity)
}

pub fn validate_did(did: &str) -> AgencyClientResult<String> {
    trace!("validate_did >>> did: {}", did);
    if is_fully_qualified(did) {
        Ok(did.to_string())
    } else {
        let check_did = String::from(did);
        match bs58::decode(check_did.clone()).into_vec() {
            Ok(ref x) if x.len() == 16 => Ok(check_did),
            Ok(x) => Err(ErrorAgencyClient::from_msg(
                ErrorKindAgencyClient::InvalidDid,
                format!("Invalid DID length, expected 16 bytes, decoded {} bytes", x.len()),
            )),
            Err(err) => {
                return Err(ErrorAgencyClient::from_msg(
                    ErrorKindAgencyClient::NotBase58,
                    format!("DID is not valid base58, details: {}", err),
                ));
            }
        }
    }
}

pub fn validate_verkey(verkey: &str) -> AgencyClientResult<String> {
    let check_verkey = String::from(verkey);
    match bs58::decode(check_verkey.clone()).into_vec() {
        Ok(ref x) if x.len() == 32 => Ok(check_verkey),
        Ok(x) => Err(ErrorAgencyClient::from_msg(
            ErrorKindAgencyClient::InvalidVerkey,
            format!("Invalid verkey length, expected 32 bytes, decoded {} bytes", x.len()),
        )),
        Err(err) => Err(ErrorAgencyClient::from_msg(
            ErrorKindAgencyClient::NotBase58,
            format!("Verkey is not valid base58, details: {}", err),
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::error::ErrorKindAgencyClient;

    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_did_is_b58_and_valid_length() {
        let to_did = "8XFh8yBzrpJQmNyZzgoTqB";
        match validate_did(&to_did) {
            Err(_) => panic!("Should be valid did"),
            Ok(x) => assert_eq!(x, to_did.to_string()),
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_did_is_b58_but_invalid_length() {
        let to_did = "8XFh8yBzrpJQmNyZzgoT";
        match validate_did(&to_did) {
            Err(x) => assert_eq!(x.kind(), ErrorKindAgencyClient::InvalidDid),
            Ok(_) => panic!("Should be invalid did"),
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_validate_did_with_non_base58() {
        let to_did = "8*Fh8yBzrpJQmNyZzgoTqB";
        match validate_did(&to_did) {
            Err(x) => assert_eq!(x.kind(), ErrorKindAgencyClient::NotBase58),
            Ok(_) => panic!("Should be invalid did"),
        }
    }

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
            Err(x) => assert_eq!(x.kind(), ErrorKindAgencyClient::InvalidVerkey),
            Ok(_) => panic!("Should be invalid verkey"),
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_validate_verkey_with_non_base58() {
        let verkey = "*kVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A";
        match validate_verkey(&verkey) {
            Err(x) => assert_eq!(x.kind(), ErrorKindAgencyClient::NotBase58),
            Ok(_) => panic!("Should be invalid verkey"),
        }
    }
}
