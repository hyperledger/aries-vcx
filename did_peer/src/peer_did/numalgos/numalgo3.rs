use did_parser::Did;

use super::traits::{Numalgo, ToNumalgo3};
use crate::{error::DidPeerError, peer_did::PeerDid};

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Numalgo3;

impl Numalgo for Numalgo3 {
    const NUMALGO_CHAR: char = '3';
}

impl ToNumalgo3 for Numalgo3 {
    fn to_numalgo3(did: &Did) -> Result<PeerDid<Numalgo3>, DidPeerError> {
        Ok(PeerDid::from_parts(did.to_owned(), Self))
    }
}
