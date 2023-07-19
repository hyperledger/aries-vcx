use did_parser::Did;

use crate::error::DidPeerError;

use super::numalgos::NumalgoKind;

pub fn parse_numalgo(did: &Did) -> Result<NumalgoKind, DidPeerError> {
    did.id()
        .chars()
        .nth(0)
        .ok_or_else(|| DidPeerError::DidValidationError(format!("Invalid did: {}", did.did())))?
        .try_into()
}
