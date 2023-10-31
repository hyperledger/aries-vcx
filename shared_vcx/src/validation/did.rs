use bs58;
use regex::Regex;

use crate::errors::validation::{ValidationError, ValidationErrorKind, ValidationResult};

lazy_static! {
    pub static ref REGEX: Regex =
        Regex::new("did:([a-z0-9]+):([a-zA-Z0-9:.-_]*)").expect("unexpected regex error occurred.");
}

pub fn is_fully_qualified(entity: &str) -> bool {
    REGEX.is_match(entity)
}

pub fn validate_did(did: &str) -> ValidationResult<String> {
    if is_fully_qualified(did) {
        Ok(did.to_string())
    } else {
        let check_did = String::from(did);
        match bs58::decode(check_did.clone()).into_vec() {
            Ok(ref x) if x.len() == 16 => Ok(check_did),
            Ok(x) => Err(ValidationError::from_msg(
                ValidationErrorKind::InvalidDid,
                format!(
                    "Invalid DID length, expected 16 bytes, decoded {} bytes",
                    x.len()
                ),
            )),
            Err(err) => Err(ValidationError::from_msg(
                ValidationErrorKind::NotBase58,
                format!("DID is not valid base58, details: {err}"),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_did_is_b58_and_valid_length() {
        let to_did = "8XFh8yBzrpJQmNyZzgoTqB";
        match validate_did(to_did) {
            Err(_) => panic!("Should be valid did"),
            Ok(x) => assert_eq!(x, to_did.to_string()),
        }
    }

    #[test]
    fn test_did_is_b58_but_invalid_length() {
        let to_did = "8XFh8yBzrpJQmNyZzgoT";
        match validate_did(to_did) {
            Err(x) => assert_eq!(x.kind(), ValidationErrorKind::InvalidDid),
            Ok(_) => panic!("Should be invalid did"),
        }
    }

    #[test]
    fn test_validate_did_with_non_base58() {
        let to_did = "8*Fh8yBzrpJQmNyZzgoTqB";
        match validate_did(to_did) {
            Err(x) => assert_eq!(x.kind(), ValidationErrorKind::NotBase58),
            Ok(_) => panic!("Should be invalid did"),
        }
    }
}
