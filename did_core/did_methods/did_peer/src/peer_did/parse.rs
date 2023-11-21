use did_parser::Did;

use crate::{error::DidPeerError, peer_did::numalgos::kind::NumalgoKind};

pub fn parse_numalgo(did: &Did) -> Result<NumalgoKind, DidPeerError> {
    did.id()
        .chars()
        .next()
        .ok_or_else(|| DidPeerError::DidValidationError(format!("Invalid did: {}", did.did())))?
        .try_into()
}
