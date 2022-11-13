use crate::error::prelude::*;

use rust_base58::FromBase58;

pub fn validate_verkey(verkey: &str) -> MessagesResult<String> {
    let check_verkey = String::from(verkey);
    match check_verkey.from_base58() {
        Ok(ref x) if x.len() == 32 => Ok(check_verkey),
        Ok(_) => Err(MessagesError::from_msg(MesssagesErrorKind::InvalidVerkey, "Invalid Verkey length")),
        Err(x) => Err(MessagesError::from_msg(
            MesssagesErrorKind::NotBase58,
            format!("Invalid Verkey: {}", x),
        )),
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use crate::utils::devsetup::SetupEmpty;

    use super::*;

    #[test]
    fn test_verkey_is_b58_and_valid_length() {
        let _setup = SetupEmpty::init();

        let verkey = "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A";
        match validate_verkey(&verkey) {
            Err(_) => panic!("Should be valid verkey"),
            Ok(x) => assert_eq!(x, verkey),
        }
    }

    #[test]
    fn test_verkey_is_b58_but_invalid_length() {
        let _setup = SetupEmpty::init();

        let verkey = "8XFh8yBzrpJQmNyZzgoT";
        match validate_verkey(&verkey) {
            Err(x) => assert_eq!(x.kind(), MesssagesErrorKind::InvalidVerkey),
            Ok(_) => panic!("Should be invalid verkey"),
        }
    }

    #[test]
    fn test_validate_verkey_with_non_base58() {
        let _setup = SetupEmpty::init();

        let verkey = "*kVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A";
        match validate_verkey(&verkey) {
            Err(x) => assert_eq!(x.kind(), MesssagesErrorKind::NotBase58),
            Ok(_) => panic!("Should be invalid verkey"),
        }
    }
}
