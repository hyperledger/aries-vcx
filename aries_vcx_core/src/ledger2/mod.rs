use async_trait::async_trait;

use crate::errors::error::VcxCoreResult;

#[async_trait]
pub trait Ledger {
    type Request: Send + Sync;

    type Response: Send + Sync;

    async fn submit<R>(&self, request: Self::Request) -> VcxCoreResult<R::Response>
    where
        R: LedgerRequest<Self>;
}

pub trait LedgerRequest<L: Ledger + ?Sized> {
    type RequestParams<'a>;

    type Response;

    fn into_ledger_request(self, params: Self::RequestParams<'_>) -> VcxCoreResult<L::Request>;

    fn from_ledger_response(response: L::Response) -> VcxCoreResult<Self::Response>
    where
        Self: Sized;
}
