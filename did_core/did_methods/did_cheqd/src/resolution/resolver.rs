use async_trait::async_trait;
use did_resolver::{
    did_doc::schema::did_doc::DidDocument,
    did_parser_nom::Did,
    error::GenericError,
    traits::resolvable::{resolution_output::DidResolutionOutput, DidResolvable},
};

use crate::error::DidCheqdResult;

pub struct DidCheqdResolver;

#[async_trait]
impl DidResolvable for DidCheqdResolver {
    type DidResolutionOptions = ();

    async fn resolve(
        &self,
        did: &Did,
        _: &Self::DidResolutionOptions,
    ) -> Result<DidResolutionOutput, GenericError> {
        let doc = self.resolve_did(did).await?;
        Ok(DidResolutionOutput::builder(doc).build())
    }
}

impl DidCheqdResolver {
    pub async fn resolve_did(&self, _did: &Did) -> DidCheqdResult<DidDocument> {
        todo!()
    }
}
