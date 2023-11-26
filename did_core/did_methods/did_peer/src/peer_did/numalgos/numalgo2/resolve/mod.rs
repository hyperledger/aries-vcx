mod helpers;

use did_doc::schema::did_doc::{DidDocument, DidDocumentBuilder};
use did_doc_sov::extra_fields::ExtraFieldsSov;
use did_parser::Did;

use self::helpers::process_elements;
use crate::{error::DidPeerError, resolver::options::PublicKeyEncoding};

pub fn resolve_numalgo2<T>(
    did: T,
    public_key_encoding: PublicKeyEncoding,
) -> Result<DidDocumentBuilder, DidPeerError>
    where
        T: Into<Did>,
{
    let did: Did = did.into();
    let mut did_doc_builder: DidDocumentBuilder = DidDocument::builder(did.clone());
    did_doc_builder = process_elements(did_doc_builder, &did, public_key_encoding)?;
    Ok(did_doc_builder)
}
