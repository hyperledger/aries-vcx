use did_parser::Did;

use super::numalgos::NumalgoKind;
use crate::error::DidPeerError;

pub fn parse_numalgo(did: &Did) -> Result<NumalgoKind, DidPeerError> {
    did.id()
        .chars()
        .next()
        .ok_or_else(|| DidPeerError::DidValidationError(format!("Invalid did: {}", did.did())))?
        .try_into()
}
