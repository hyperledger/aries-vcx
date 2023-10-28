use async_trait::async_trait;
use did_doc_sov::extra_fields::ExtraFieldsSov;
use did_parser::Did;
use did_resolver::{
    error::GenericError,
    traits::resolvable::{
        resolution_metadata::DidResolutionMetadata, resolution_options::DidResolutionOptions,
        resolution_output::DidResolutionOutput, DidResolvable,
    },
};

use super::options::ExtraFieldsOptions;
use crate::{
    error::DidPeerError,
    peer_did::{generic::AnyPeerDid, numalgos::numalgo2::resolve::resolve_numalgo2},
};

pub struct PeerDidResolver;

#[async_trait]
impl DidResolvable for PeerDidResolver {
    type ExtraFieldsService = ExtraFieldsSov;
    type ExtraFieldsOptions = ExtraFieldsOptions;

    async fn resolve(
        &self,
        did: &Did,
        options: &DidResolutionOptions<Self::ExtraFieldsOptions>,
    ) -> Result<DidResolutionOutput<Self::ExtraFieldsService>, GenericError> {
        let peer_did = AnyPeerDid::parse(did.to_owned())?;
        match peer_did {
            AnyPeerDid::Numalgo2(peer_did) => {
                let did_doc =
                    resolve_numalgo2(peer_did.did(), options.extra().public_key_encoding())?
                        .add_also_known_as(peer_did.to_numalgo3()?.to_string().parse()?)
                        .build();
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
