use async_trait::async_trait;
use indy_vdr::pool::PreparedRequest;

use crate::errors::error::VcxLedgerResult;

pub mod vdr_ledger;
#[cfg(feature = "vdr_proxy_ledger")]
pub mod vdr_proxy;

#[async_trait]
pub trait RequestSubmitter: Send + Sync {
    async fn submit(&self, request: PreparedRequest) -> VcxLedgerResult<String>;
}
