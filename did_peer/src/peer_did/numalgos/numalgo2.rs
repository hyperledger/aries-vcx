use did_doc::schema::did_doc::DidDocument;
use did_doc_sov::extra_fields::ExtraFieldsSov;
use did_parser::Did;

use super::traits::{Numalgo, ResolvableNumalgo};
use crate::{
    error::DidPeerError, numalgos::numalgo2::resolve_numalgo2,
    peer_did_resolver::options::PublicKeyEncoding,
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
        resolve_numalgo2(did, public_key_encoding).map(|builder| builder.build())
    }
}
