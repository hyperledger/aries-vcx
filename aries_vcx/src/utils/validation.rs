use bs58;
use messages::actors::Actors;
use openssl::bn::BigNum;

use crate::{errors::error::prelude::*, utils::qualifier};

pub fn validate_did(did: &str) -> VcxResult<String> {
    if qualifier::is_fully_qualified(did) {
        Ok(did.to_string())
    } else {
        let check_did = String::from(did);
        match bs58::decode(check_did.clone()).into_vec() {
            Ok(ref x) if x.len() == 16 => Ok(check_did),
            Ok(x) => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidDid,
                format!("Invalid DID length, expected 16 bytes, decoded {} bytes", x.len()),
            )),
            Err(err) => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::NotBase58,
                format!("DID is not valid base58, details: {}", err),
            )),
        }
    }
}

pub fn validate_nonce(nonce: &str) -> VcxResult<String> {
    let nonce =
        BigNum::from_dec_str(nonce).map_err(|err| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidNonce, err))?;
    if nonce.num_bits() > 80 {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidNonce,
            "Invalid Nonce length",
        ));
    }
    Ok(nonce.to_string())
}

pub fn validate_key_delegate(delegate: &str) -> VcxResult<String> {
    //todo: find out what needs to be validated for key_delegate
    let check_delegate = String::from(delegate);
    Ok(check_delegate)
}

pub fn validate_actors(actors: &str) -> VcxResult<Vec<Actors>> {
    ::serde_json::from_str(actors)
        .map_err(|err| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidOption, format!("Invalid actors: {:?}", err)))
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use super::*;
    use crate::utils::devsetup::SetupDefaults;

    #[test]
    fn test_did_is_b58_and_valid_length() {
        let _setup = SetupDefaults::init();

        let to_did = "8XFh8yBzrpJQmNyZzgoTqB";
        match validate_did(&to_did) {
            Err(_) => panic!("Should be valid did"),
            Ok(x) => assert_eq!(x, to_did.to_string()),
        }
    }

    #[test]
    fn test_did_is_b58_but_invalid_length() {
        let _setup = SetupDefaults::init();

        let to_did = "8XFh8yBzrpJQmNyZzgoT";
        match validate_did(&to_did) {
            Err(x) => assert_eq!(x.kind(), AriesVcxErrorKind::InvalidDid),
            Ok(_) => panic!("Should be invalid did"),
        }
    }

    #[test]
    fn test_validate_did_with_non_base58() {
        let _setup = SetupDefaults::init();

        let to_did = "8*Fh8yBzrpJQmNyZzgoTqB";
        match validate_did(&to_did) {
            Err(x) => assert_eq!(x.kind(), AriesVcxErrorKind::NotBase58),
            Ok(_) => panic!("Should be invalid did"),
        }
    }
}
