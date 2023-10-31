use async_trait::async_trait;

use crate::errors::error::VcxCoreResult;

#[async_trait]
pub trait Ledger {
    type Request: Send + Sync;

    type Response: Send + Sync;

    async fn submit<R>(&self, request: R) -> VcxCoreResult<R::Output>
    where
        R: IntoLedgerRequest<Self>;
}

pub trait IntoLedgerRequest<L: Ledger + ?Sized> {
    type Output;

    fn into_ledger_request(self) -> VcxCoreResult<L::Request>;

    fn from_ledger_response(response: L::Response) -> VcxCoreResult<Self::Output>
    where
        Self: Sized;
}
