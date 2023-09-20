use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::ResponseCacher;
use crate::errors::error::VcxCoreResult;

pub struct NoopResponseCacher {}

impl NoopResponseCacher {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ResponseCacher for NoopResponseCacher {
    type Options = ();

    async fn put<S, T>(&self, _id: S, _obj: T) -> VcxCoreResult<()>
    where
        S: ToString + Send,
        T: Serialize + for<'de> Deserialize<'de> + Send,
    {
        Ok(())
    }

    async fn get<S, T>(&self, _id: S, _opt: Option<Self::Options>) -> VcxCoreResult<Option<T>>
    where
        S: ToString + Send,
        T: Serialize + for<'de> Deserialize<'de> + Send,
    {
        Ok(None)
    }
}
