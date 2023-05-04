use std::sync::Arc;

use async_trait::async_trait;
use did_resolver::{
    did_parser::Did,
    error::GenericError,
    shared_types::media_type::MediaType,
    traits::resolvable::{
        resolution_options::DidResolutionOptions, resolution_output::DidResolutionOutput,
        DidResolvable,
    },
};

use crate::{error::DidSovError, reader::AttrReader};

use super::utils::{is_valid_sovrin_did_id, ledger_response_to_ddo};

pub struct DidSovResolver {
    ledger: Arc<dyn AttrReader>,
}

impl DidSovResolver {
    pub fn new(ledger: Arc<dyn AttrReader>) -> Self {
        DidSovResolver { ledger }
    }
}

#[async_trait]
impl DidResolvable for DidSovResolver {
    async fn resolve(
        &self,
        parsed_did: &Did,
        options: &DidResolutionOptions,
    ) -> Result<DidResolutionOutput, GenericError> {
        if let Some(accept) = options.accept() {
            if accept != &MediaType::DidJson {
                return Err(Box::new(DidSovError::RepresentationNotSupported(
                    accept.to_string(),
                )));
            }
        }
        if parsed_did.method() != "sov" {
            return Err(Box::new(DidSovError::MethodNotSupported(
                parsed_did.method().to_string(),
            )));
        }
        if !is_valid_sovrin_did_id(parsed_did.id()) {
            return Err(Box::new(DidSovError::InvalidDid(
                parsed_did.id().to_string(),
            )));
        }
        let did = parsed_did.did();
        let ledger_response = self.ledger.get_attr(did, "endpoint").await?;
        ledger_response_to_ddo(did, &ledger_response)
            .await
            .map_err(|err| err.into())
    }
}
