use std::sync::Arc;

use async_trait::async_trait;
use indy_vdr::pool::PreparedRequest;
use indy_vdr_proxy_client::VdrProxyClient;

use crate::errors::error::VcxCoreResult;

use super::RequestSubmitter;

pub struct VdrProxySubmitter {
    client: Arc<VdrProxyClient>,
}

impl VdrProxySubmitter {
    pub fn new(client: Arc<VdrProxyClient>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl RequestSubmitter for VdrProxySubmitter {
    async fn submit(&self, request: PreparedRequest) -> VcxCoreResult<String> {
        self.client.post(request).await.map_err(|e| e.into())
    }
}
