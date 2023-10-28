use did_parser::Did;

use super::regex::PEER_DID_REGEX;
use crate::error::DidPeerError;

pub fn validate(did: &Did) -> Result<(), DidPeerError> {
    if !PEER_DID_REGEX.is_match(did.did()) {
        Err(DidPeerError::DidValidationError(format!(
            "Invalid did: {} because it's not matching peer did regex {}",
            did.did(),
            *PEER_DID_REGEX
        )))
    } else {
        Ok(())
    }
}
