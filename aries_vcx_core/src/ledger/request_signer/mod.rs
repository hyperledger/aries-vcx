pub mod base_wallet;

use async_trait::async_trait;
use indy_vdr::pool::PreparedRequest;

use crate::errors::error::VcxCoreResult;

#[async_trait]
pub trait RequestSigner: Send + Sync {
    async fn sign(&self, did: &str, request: &PreparedRequest) -> VcxCoreResult<Vec<u8>>;
}
