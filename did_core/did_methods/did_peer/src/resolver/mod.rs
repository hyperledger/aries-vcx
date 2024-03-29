use async_trait::async_trait;
use did_doc::schema::did_doc::DidDocument;
use did_parser_nom::Did;
use did_resolver::{
    error::GenericError,
    traits::resolvable::{
        resolution_metadata::DidResolutionMetadata, resolution_output::DidResolutionOutput,
        DidResolvable,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    error::DidPeerError, peer_did::generic::AnyPeerDid, resolver::options::PublicKeyEncoding,
};

pub mod options;

#[derive(Default)]
pub struct PeerDidResolver;

impl PeerDidResolver {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct PeerDidResolutionOptions {
    pub encoding: Option<PublicKeyEncoding>,
}
#[async_trait]
impl DidResolvable for PeerDidResolver {
    type DidResolutionOptions = PeerDidResolutionOptions;

    async fn resolve(
        &self,
        did: &Did,
        options: &Self::DidResolutionOptions,
    ) -> Result<DidResolutionOutput, GenericError> {
        let peer_did = AnyPeerDid::parse(did.to_owned())?;
        let did_doc = match peer_did {
            AnyPeerDid::Numalgo2(peer_did) => {
                let encoding = options.encoding.unwrap_or(PublicKeyEncoding::Multibase);
                let mut did_doc: DidDocument = peer_did.to_did_doc_builder(encoding)?;
                did_doc.add_also_known_as(peer_did.to_numalgo3()?.to_string().parse()?);
                did_doc
            }
            AnyPeerDid::Numalgo4(peer_did) => peer_did.resolve_did_doc()?,
            n => return Err(Box::new(DidPeerError::UnsupportedNumalgo(n.numalgo()))),
        };
        let resolution_metadata = DidResolutionMetadata::builder()
            .content_type("application/did+json".to_string())
            .build();
        let builder =
            DidResolutionOutput::builder(did_doc).did_resolution_metadata(resolution_metadata);
        Ok(builder.build())
    }
}
