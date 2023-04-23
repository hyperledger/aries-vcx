use std::{num::NonZeroUsize, sync::Arc};

use aries_vcx_core::ledger::base_ledger::BaseLedger;
use async_trait::async_trait;
use did_resolver::{
    did_parser::ParsedDID,
    error::GenericError,
    traits::resolvable::{
        resolution_options::DIDResolutionOptions, resolution_output::DIDResolutionOutput,
        DIDResolvable, DIDResolvableMut,
    },
};
use lru::LruCache;

use crate::error::DIDSovError;

use super::utils::{is_valid_sovrin_did_id, resolve_ddo};

pub struct DIDSovResolver {
    ledger: Arc<dyn BaseLedger>,
    cache: LruCache<String, Arc<DIDResolutionOutput>>,
}

impl DIDSovResolver {
    pub fn new(ledger: Arc<dyn BaseLedger>, cache_size: NonZeroUsize) -> Self {
        DIDSovResolver {
            ledger,
            cache: LruCache::new(cache_size),
        }
    }
}

#[async_trait]
impl DIDResolvable for DIDSovResolver {
    async fn resolve(
        &self,
        parsed_did: &ParsedDID,
        options: &DIDResolutionOptions,
    ) -> Result<DIDResolutionOutput, GenericError> {
        if let Some(accept) = options.accept() {
            if accept != "application/did+json" {
                return Err(Box::new(DIDSovError::RepresentationNotSupported(
                    accept.to_string(),
                )));
            }
        }
        if parsed_did.method() != "sov" {
            return Err(Box::new(DIDSovError::MethodNotSupported(
                parsed_did.method().to_string(),
            )));
        }
        if !is_valid_sovrin_did_id(parsed_did.id()) {
            return Err(Box::new(DIDSovError::InvalidDID(
                parsed_did.id().to_string(),
            )));
        }
        let did = parsed_did.did();
        let ledger_response = self.ledger.get_attr(did, "endpoint").await?;
        resolve_ddo(did, &ledger_response)
            .await
            .map_err(|err| err.into())
    }
}

#[async_trait]
impl DIDResolvableMut for DIDSovResolver {
    async fn resolve_mut(
        &mut self,
        parsed_did: &ParsedDID,
        options: &DIDResolutionOptions,
    ) -> Result<DIDResolutionOutput, GenericError> {
        let did = parsed_did.did();
        if let Some(resolution_output) = self.cache.get(did) {
            return Ok((**resolution_output).clone());
        }
        let resolution_output = self.resolve(parsed_did, options).await?;
        self.cache
            .put(did.to_string(), Arc::new(resolution_output.clone()));
        Ok(resolution_output)
    }
}
