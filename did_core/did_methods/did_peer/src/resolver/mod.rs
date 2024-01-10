use async_trait::async_trait;
use did_doc::schema::did_doc::DidDocumentBuilder;
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
    error::DidPeerError,
    peer_did::{generic::AnyPeerDid, numalgos::numalgo2::Numalgo2, PeerDid},
    resolver::options::PublicKeyEncoding,
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

impl PeerDidResolver {
    pub async fn resolve_peerdid2(
        peer_did: &PeerDid<Numalgo2>,
        public_key_encoding: PublicKeyEncoding,
    ) -> Result<DidResolutionOutput, GenericError> {
        let builder: DidDocumentBuilder = peer_did.to_did_doc_builder(public_key_encoding)?;
        let did_doc = builder
            .add_also_known_as(peer_did.to_numalgo3()?.to_string().parse()?)
            .build();
        let resolution_metadata = DidResolutionMetadata::builder()
            .content_type("application/did+json".to_string())
            .build();
        let builder =
            DidResolutionOutput::builder(did_doc).did_resolution_metadata(resolution_metadata);
        Ok(builder.build())
    }
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
                Self::resolve_peerdid2(
                    &peer_did,
                    options.encoding.unwrap_or(PublicKeyEncoding::Multibase),
                )
                .await
            }
            n => Err(Box::new(DidPeerError::UnsupportedNumalgo(n.numalgo()))),
        }
    }
}
