mod helpers;

use did_doc::schema::did_doc::{DidDocument, DidDocumentBuilder};
use did_doc_sov::extra_fields::ExtraFieldsSov;
use did_parser::Did;

use crate::{error::DidPeerError, peer_did_resolver::options::PublicKeyEncoding};

use self::helpers::process_elements;

pub fn resolve_numalgo2(
    did: &Did,
    public_key_encoding: PublicKeyEncoding,
) -> Result<DidDocument<ExtraFieldsSov>, DidPeerError> {
    let mut did_doc_builder: DidDocumentBuilder<ExtraFieldsSov> = DidDocument::builder(did.to_owned());

    did_doc_builder = process_elements(did_doc_builder, did, public_key_encoding)?;

    Ok(did_doc_builder.build())
}
