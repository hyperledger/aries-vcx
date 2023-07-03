mod helpers;

use did_doc::schema::did_doc::DidDocument;
use did_doc_sov::extra_fields::ExtraFieldsSov;

use crate::{
    error::DidPeerError,
    peer_did::{numalgos::numalgo2::Numalgo2, peer_did::PeerDid},
};

use self::helpers::{append_encoded_key_segments, append_encoded_service_segment};

pub fn generate_numalgo2(did_document: DidDocument<ExtraFieldsSov>) -> Result<PeerDid<Numalgo2>, DidPeerError> {
    let mut did = String::from("did:peer:2");

    did = append_encoded_key_segments(did, &did_document)?;
    did = append_encoded_service_segment(did, &did_document)?;

    PeerDid::<Numalgo2>::parse(did)
}
