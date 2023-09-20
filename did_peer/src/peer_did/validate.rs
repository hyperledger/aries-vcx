use did_parser::Did;

use super::regex::PEER_DID_REGEX;
use crate::error::DidPeerError;

pub fn validate(did: &Did) -> Result<(), DidPeerError> {
    if !PEER_DID_REGEX.is_match(did.did()) {
        Err(DidPeerError::DidValidationError(format!(
            "Invalid did: {}",
            did.did()
        )))
    } else {
        Ok(())
    }
}
