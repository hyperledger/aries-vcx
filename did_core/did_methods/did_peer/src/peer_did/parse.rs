use did_parser_nom::Did;

use crate::{error::DidPeerError, peer_did::numalgos::kind::NumalgoKind};

pub fn parse_numalgo(did: &Did) -> Result<NumalgoKind, DidPeerError> {
    log::info!("did.id() >> {}", did.id());
    did.id()
        .chars()
        .next()
        .ok_or_else(|| {
            DidPeerError::DidValidationError(format!(
                "Invalid peer did: {} because numalgo couldn't be parsed",
                did.did()
            ))
        })?
        .try_into()
}
