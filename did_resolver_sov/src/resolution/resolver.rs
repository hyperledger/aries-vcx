use async_trait::async_trait;
use did_doc_sov::extra_fields::ExtraFieldsSov;
use did_resolver::{
    did_parser::Did,
    error::GenericError,
    shared_types::media_type::MediaType,
    traits::resolvable::{
        resolution_options::DidResolutionOptions, resolution_output::DidResolutionOutput,
        DidResolvable,
    },
};
use serde_json::Value;

use super::utils::{is_valid_sovrin_did_id, ledger_response_to_ddo};
use crate::{
    error::{parsing::ParsingErrorSource, DidSovError},
    reader::AttrReader,
};

pub struct DidSovResolver<T: AttrReader> {
    ledger: T,
}

impl<T: AttrReader> DidSovResolver<T> {
    pub fn new(ledger: T) -> Self {
        DidSovResolver { ledger }
    }
}

#[async_trait]
impl<T: AttrReader> DidResolvable for DidSovResolver<T> {
    type ExtraFieldsService = ExtraFieldsSov;
    type ExtraFieldsOptions = ();

    async fn resolve(
        &self,
        parsed_did: &Did,
        options: &DidResolutionOptions<()>,
    ) -> Result<DidResolutionOutput<Self::ExtraFieldsService>, GenericError> {
        if let Some(accept) = options.accept() {
            if accept != &MediaType::DidJson {
                return Err(Box::new(DidSovError::RepresentationNotSupported(
                    accept.to_string(),
                )));
            }
        }
        let method = parsed_did.method().ok_or_else(|| {
            DidSovError::InvalidDid("Attempted to resolve unqualified did".to_string())
        })?;
        if method != "sov" {
            return Err(Box::new(DidSovError::MethodNotSupported(
                method.to_string(),
            )));
        }
        if !is_valid_sovrin_did_id(parsed_did.id()) {
            return Err(Box::new(DidSovError::InvalidDid(
                parsed_did.id().to_string(),
            )));
        }
        let did = parsed_did.did();
        let ledger_response = self.ledger.get_attr(did, "endpoint").await?;
        let verkey = self.get_verkey(did).await?;
        ledger_response_to_ddo(did, &ledger_response, verkey)
            .await
            .map_err(|err| err.into())
    }
}

impl<T: AttrReader> DidSovResolver<T> {
    async fn get_verkey(&self, did: &str) -> Result<String, DidSovError> {
        let nym_response = self.ledger.get_nym(did).await?;
        let nym_json: Value = serde_json::from_str(&nym_response)?;
        let nym_data = nym_json["result"]["data"]
            .as_str()
            .ok_or(DidSovError::ParsingError(
                ParsingErrorSource::LedgerResponseParsingError(
                    "Failed to parse nym data".to_string(),
                ),
            ))?;
        let nym_data: Value = serde_json::from_str(nym_data)?;
        let verkey = nym_data["verkey"]
            .as_str()
            .ok_or(DidSovError::ParsingError(
                ParsingErrorSource::LedgerResponseParsingError(
                    "Failed to parse verkey from nym data".to_string(),
                ),
            ))?;
        Ok(verkey.to_string())
    }
}
