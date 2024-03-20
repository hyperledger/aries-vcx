pub mod in_memory;
pub mod noop;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::errors::error::VcxLedgerResult;

#[async_trait]
pub trait ResponseCacher: Send + Sync {
    type Options: Send + Sync;

    async fn put<S, T>(&self, id: S, obj: T) -> VcxLedgerResult<()>
    where
        S: ToString + Send,
        T: Serialize + for<'de> Deserialize<'de> + Send;

    async fn get<S, T>(&self, id: S, opt: Option<Self::Options>) -> VcxLedgerResult<Option<T>>
    where
        S: ToString + Send,
        T: Serialize + for<'de> Deserialize<'de> + Send;
}
