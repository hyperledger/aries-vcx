use did_doc::schema::did_doc::DidDocument;
use did_doc_sov::extra_fields::ExtraFieldsSov;
use did_parser::Did;

use crate::{
    error::DidPeerError,
    numalgos::{numalgo2::resolve_numalgo2, numalgo3::generate_numalgo3},
    peer_did::peer_did::PeerDid,
    peer_did_resolver::options::PublicKeyEncoding,
};

use super::{
    numalgo3::Numalgo3,
    traits::{Numalgo, ResolvableNumalgo, ToNumalgo3},
};

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Numalgo2;

impl Numalgo for Numalgo2 {
    const NUMALGO_CHAR: char = '2';
}

impl ResolvableNumalgo for Numalgo2 {
    fn resolve(
        &self,
        did: &Did,
        public_key_encoding: PublicKeyEncoding,
    ) -> Result<DidDocument<ExtraFieldsSov>, DidPeerError> {
        resolve_numalgo2(did, public_key_encoding)
    }
}

impl ToNumalgo3 for Numalgo2 {
    fn to_numalgo3(did: &Did) -> Result<PeerDid<Numalgo3>, DidPeerError> {
        generate_numalgo3(did)
    }
}
