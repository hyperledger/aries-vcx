use did_doc::schema::did_doc::DidDocument;
use did_doc_sov::extra_fields::ExtraFieldsSov;
use did_parser::Did;
use encoding::{append_encoded_key_segments, append_encoded_service_segment};
use sha256::digest;

use crate::{
    error::DidPeerError,
    peer_did::{
        numalgos::{
            numalgo2::resolve::resolve_numalgo2, numalgo3::Numalgo3, Numalgo, ResolvableNumalgo,
        },
        FromDidDoc, PeerDid,
    },
    resolver::options::PublicKeyEncoding,
};

mod encoding;
mod purpose;
pub mod resolve;
mod service_abbreviated;
mod verification_method;

impl FromDidDoc for Numalgo2 {
    fn from_did_doc(
        did_document: DidDocument,
    ) -> Result<PeerDid<Numalgo2>, DidPeerError> {
        let mut did = String::from("did:peer:2");
        did = append_encoded_key_segments(did, &did_document)?;
        did = append_encoded_service_segment(did, &did_document)?;
        PeerDid::<Numalgo2>::parse(did)
    }
}

impl PeerDid<Numalgo2> {
    pub fn to_numalgo3(&self) -> Result<PeerDid<Numalgo3>, DidPeerError> {
        let numalgoless_id = self.did().id().chars().skip(2).collect::<String>();
        let numalgoless_id_hashed = digest(numalgoless_id);
        PeerDid::<Numalgo3>::parse(format!("did:peer:3.{}", numalgoless_id_hashed))
    }
}

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
    ) -> Result<DidDocument, DidPeerError> {
        resolve_numalgo2(did, public_key_encoding).map(|builder| builder.build())
    }
}
