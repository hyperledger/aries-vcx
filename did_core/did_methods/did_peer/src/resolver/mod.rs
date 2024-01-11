use async_trait::async_trait;
use did_parser::Did;
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
        match peer_did {
            AnyPeerDid::Numalgo2(peer_did) => {
                let did_doc =
                    peer_did.resolve(options.encoding.unwrap_or(PublicKeyEncoding::Base58))?;
                let resolution_metadata = DidResolutionMetadata::builder()
                    .content_type("application/did+json".to_string())
                    .build();
                let builder = DidResolutionOutput::builder(did_doc)
                    .did_resolution_metadata(resolution_metadata);
                Ok(builder.build())
            }
            n => Err(Box::new(DidPeerError::UnsupportedNumalgo(n.numalgo()))),
        }
    }
}
