use did_parser::Did;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::error::DidPeerError;

static GROUP_NUMALGO_0_AND_1: &str = r"([01](z)([1-9a-km-zA-HJ-NP-Z]{5,200}))";
static GROUP_NUMALGO_2: &str =
    r"(2((.[AEVID](z)([1-9a-km-zA-HJ-NP-Z]{5,200}))*(.(S)[0-9a-zA-Z=]*)?))";
static GROUP_NUMALGO_3: &str = r"(3\.[0-9a-fA-F]{64})";

pub static PEER_DID_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(
        r"^did:peer:({GROUP_NUMALGO_0_AND_1}|{GROUP_NUMALGO_2}|{GROUP_NUMALGO_3})$"
    ))
    .unwrap()
});

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
