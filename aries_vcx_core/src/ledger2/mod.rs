mod indy;

use async_trait::async_trait;

use crate::errors::error::VcxCoreResult;

#[async_trait]
pub trait Ledger {
    type Request: Send + Sync;

    type Response: Send + Sync;

    async fn submit<R>(&self, request: Self::Request) -> VcxCoreResult<R>
    where
        R: LedgerRequest<Self>;
}

pub trait LedgerRequest<L: Ledger + ?Sized> {
    fn into_ledger_request(self) -> VcxCoreResult<L::Request>;

    fn as_ledger_request(&self) -> VcxCoreResult<L::Request>;

    fn from_ledger_response(response: L::Response) -> VcxCoreResult<Self>
    where
        Self: Sized;
}
