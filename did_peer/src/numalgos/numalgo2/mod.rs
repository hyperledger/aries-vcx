mod encoding;
mod purpose;
mod resolve;
mod service_abbreviated;
mod verification_method;

use did_doc::schema::did_doc::DidDocument;
use did_doc_sov::extra_fields::ExtraFieldsSov;
pub use resolve::resolve_numalgo2;
use sha256::digest;

use crate::{
    error::DidPeerError,
    numalgos::numalgo2::encoding::{append_encoded_key_segments, append_encoded_service_segment},
    peer_did::{
        numalgos::{numalgo2::Numalgo2, numalgo3::Numalgo3},
        FromDidDoc, PeerDid,
    },
};

impl FromDidDoc for Numalgo2 {
    fn from_did_doc(
        did_document: DidDocument<ExtraFieldsSov>,
    ) -> Result<PeerDid<Numalgo2>, DidPeerError> {
        let mut did = String::from("did:peer:2");

        did = append_encoded_key_segments(did, &did_document)?;
        println!("after append_encoded_key_segments >> did: {}", did);
        did = append_encoded_service_segment(did, &did_document)?;
        println!("after append_encoded_service_segment >> did: {}", did);

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
