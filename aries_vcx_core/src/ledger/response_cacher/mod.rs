pub mod in_memory;
pub mod noop;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::errors::error::VcxCoreResult;

#[async_trait]
pub trait ResponseCacher: Send + Sync {
    type Options: Send + Sync;

    async fn put<S, T>(&self, id: S, obj: T) -> VcxCoreResult<()>
    where
        S: ToString + Send,
        T: Serialize + for<'de> Deserialize<'de> + Send;

    async fn get<S, T>(&self, id: S, opt: Option<Self::Options>) -> VcxCoreResult<Option<T>>
    where
        S: ToString + Send,
        T: Serialize + for<'de> Deserialize<'de> + Send;
}
