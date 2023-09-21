use did_doc::schema::did_doc::DidDocument;
use did_doc_sov::extra_fields::ExtraFieldsSov;

use super::{
    numalgos::{numalgo2::Numalgo2, numalgo3::Numalgo3},
    PeerDid,
};
use crate::{
    error::DidPeerError,
    numalgos::{numalgo2, numalgo3},
};

pub fn generate_numalgo2(
    did_document: DidDocument<ExtraFieldsSov>,
) -> Result<PeerDid<Numalgo2>, DidPeerError> {
    numalgo2::generate_numalgo2(did_document)
}

pub fn generate_numalgo3(
    did_document: DidDocument<ExtraFieldsSov>,
) -> Result<PeerDid<Numalgo3>, DidPeerError> {
    numalgo3::generate_numalgo3(generate_numalgo2(did_document)?.did())
}
