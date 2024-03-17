use std::{borrow::Borrow, marker::PhantomData};

use async_trait::async_trait;
use did_resolver::{
    did_parser_nom::Did,
    error::GenericError,
    traits::resolvable::{resolution_output::DidResolutionOutput, DidResolvable},
};
use serde_json::Value;

use super::utils::{is_valid_sovrin_did_id, ledger_response_to_ddo};
use crate::{
    error::{parsing::ParsingErrorSource, DidSovError},
    reader::AttrReader,
};

pub struct DidSovResolver<T, A>
where
    T: Borrow<A> + Sync + Send,
    A: AttrReader,
{
    ledger: T,
    _marker: PhantomData<A>,
}

impl<T, A> DidSovResolver<T, A>
where
    T: Borrow<A> + Sync + Send,
    A: AttrReader,
{
    // todo: Creating instance can be non-ergonomic, as compiler will ask you to specify
    //       the full type of DidSovResolver<T, A> explicitly, and the type can be quite long.
    //       Consider improving the DX in the future.
    pub fn new(ledger: T) -> Self {
        DidSovResolver {
            ledger,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<T, A> DidResolvable for DidSovResolver<T, A>
where
    T: Borrow<A> + Sync + Send,
    A: AttrReader,
{
    type DidResolutionOptions = ();

    async fn resolve(
        &self,
        parsed_did: &Did,
        _options: &Self::DidResolutionOptions,
    ) -> Result<DidResolutionOutput, GenericError> {
        log::info!("DidSovResolver::resolve >> Resolving did: {}", parsed_did);
        let method = parsed_did.method().ok_or_else(|| {
            DidSovError::InvalidDid("Attempted to resolve unqualified did".to_string())
        })?;
        if method != "sov" {
            return Err(Box::new(DidSovError::MethodNotSupported(
                method.to_string(),
            )));
        }
        if !is_valid_sovrin_did_id(parsed_did.id()) {
            return Err(Box::new(DidSovError::InvalidDid(format!(
                "Sovrin DID: {} contains invalid DID ID.",
                parsed_did.id()
            ))));
        }
        let ledger_response = self
            .ledger
            .borrow()
            .get_attr(parsed_did, "endpoint")
            .await?;
        let verkey = self.get_verkey(parsed_did).await?;
        ledger_response_to_ddo(parsed_did.did(), &ledger_response, verkey)
            .await
            .map_err(|err| err.into())
    }
}

impl<T, A> DidSovResolver<T, A>
where
    T: Borrow<A> + Sync + Send,
    A: AttrReader,
{
    async fn get_verkey(&self, did: &Did) -> Result<String, DidSovError> {
        let nym_response = self.ledger.borrow().get_nym(did).await?;
        log::info!("get_verkey >> nym_response: {}", nym_response);
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
