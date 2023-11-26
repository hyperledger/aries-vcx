mod helpers;

use did_doc::schema::did_doc::{DidDocument, DidDocumentBuilder};
use did_parser::Did;

use self::helpers::didpeer_elements_to_diddoc;
use crate::{error::DidPeerError, resolver::options::PublicKeyEncoding};

pub fn resolve_numalgo2<T>(
    did: &T,
    public_key_encoding: PublicKeyEncoding,
) -> Result<DidDocumentBuilder, DidPeerError>
where
    T: AsRef<Did>,
{
    let did_ref: &Did = did.as_ref();
    let mut did_doc_builder: DidDocumentBuilder = DidDocument::builder(did_ref.clone());
    did_doc_builder = didpeer_elements_to_diddoc(did_doc_builder, did_ref, public_key_encoding)?;
    Ok(did_doc_builder)
}
