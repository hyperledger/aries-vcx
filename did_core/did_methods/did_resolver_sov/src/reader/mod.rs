use aries_vcx_core::ledger::base_ledger::IndyLedgerRead;
use async_trait::async_trait;
use did_resolver::did_parser_nom::Did;

use crate::error::DidSovError;

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait AttrReader: Send + Sync {
    async fn get_attr(&self, target_did: &Did, attr_name: &str) -> Result<String, DidSovError>;
    async fn get_nym(&self, did: &Did) -> Result<String, DidSovError>;
}

#[async_trait]
impl<S> AttrReader for S
where
    S: IndyLedgerRead + ?Sized,
{
    async fn get_attr(&self, target_did: &Did, attr_name: &str) -> Result<String, DidSovError> {
        IndyLedgerRead::get_attr(self, target_did, attr_name)
            .await
            .map_err(|err| err.into())
    }

    async fn get_nym(&self, did: &Did) -> Result<String, DidSovError> {
        IndyLedgerRead::get_nym(self, did)
            .await
            .map_err(|err| err.into())
    }
}
