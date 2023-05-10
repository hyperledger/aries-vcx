#[cfg(feature = "vdrtools")]
pub mod indy_reader;
#[cfg(feature = "modular_libs")]
pub mod vdr_reader;

use std::sync::Arc;

use aries_vcx_core::ledger::base_ledger::BaseLedger;
use async_trait::async_trait;

use crate::error::DidSovError;

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait AttrReader: Send + Sync {
    async fn get_attr(&self, target_did: &str, attr_name: &str) -> Result<String, DidSovError>;
    async fn get_nym(&self, did: &str) -> Result<String, DidSovError>;
}

pub struct ConcreteAttrReader {
    ledger: Arc<dyn BaseLedger>,
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

impl From<Arc<dyn BaseLedger>> for ConcreteAttrReader {
    fn from(ledger: Arc<dyn BaseLedger>) -> Self {
        Self { ledger }
    }
}
