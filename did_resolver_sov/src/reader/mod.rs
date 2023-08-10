use std::sync::Arc;

use aries_vcx_core::ledger::base_ledger::IndyLedgerRead;
use async_trait::async_trait;

use crate::error::DidSovError;

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait AttrReader: Send + Sync {
    async fn get_attr(&self, target_did: &str, attr_name: &str) -> Result<String, DidSovError>;
    async fn get_nym(&self, did: &str) -> Result<String, DidSovError>;
}

pub struct ConcreteAttrReader {
    ledger: Arc<dyn IndyLedgerRead>,
}

#[async_trait]
impl AttrReader for ConcreteAttrReader {
    async fn get_attr(&self, target_did: &str, attr_name: &str) -> Result<String, DidSovError> {
        self.ledger
            .get_attr(target_did, attr_name)
            .await
            .map_err(|err| err.into())
    }

    async fn get_nym(&self, did: &str) -> Result<String, DidSovError> {
        self.ledger.get_nym(did).await.map_err(|err| err.into())
    }
}

impl From<Arc<dyn IndyLedgerRead>> for ConcreteAttrReader {
    fn from(ledger: Arc<dyn IndyLedgerRead>) -> Self {
        Self { ledger }
    }
}
