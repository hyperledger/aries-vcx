mod config;
mod options;

use std::{sync::Arc, time::Instant};

use async_trait::async_trait;
pub use config::*;
use log::info;
use lru::LruCache;
pub use options::*;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::ResponseCacher;
use crate::errors::error::VcxLedgerResult;

pub struct InMemoryResponseCacher {
    cache: Arc<Mutex<LruCache<String, (String, Instant)>>>,
    config: InMemoryResponseCacherConfig,
}

impl InMemoryResponseCacher {
    pub fn new(config: InMemoryResponseCacherConfig) -> Self {
        info!("InMemoryResponseCacher::new >> config: {config:?}");
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(config.capacity()))),
            config,
        }
    }
}

#[async_trait]
impl ResponseCacher for InMemoryResponseCacher {
    type Options = InMemoryResponseCacherOptions;

    async fn put<S, T>(&self, id: S, obj: T) -> VcxLedgerResult<()>
    where
        S: ToString + Send,
        T: Serialize + for<'de> Deserialize<'de> + Send,
    {
        let id = id.to_string();
        let obj = serde_json::to_string(&obj)?;

        let mut cache = self.cache.lock().await;
        cache.put(id, (obj, Instant::now()));
        Ok(())
    }

    async fn get<S, T>(&self, id: S, opt: Option<Self::Options>) -> VcxLedgerResult<Option<T>>
    where
        S: ToString + Send,
        T: Serialize + for<'de> Deserialize<'de> + Send,
    {
        let id = id.to_string();

        let ttl = if let Some(opt) = opt {
            opt.ttl().unwrap_or(self.config.ttl())
        } else {
            self.config.ttl()
        };

        let mut cache = self.cache.lock().await;
        match cache.get(&id) {
            Some((obj, timestamp)) => {
                if timestamp.elapsed() > ttl {
                    cache.pop(&id);
                    Ok(None)
                } else {
                    let obj: T = serde_json::from_str(obj)?;
                    Ok(Some(obj))
                }
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct TestStruct {
        field: String,
    }

    fn _cacher_config(ttl: Duration) -> InMemoryResponseCacherConfig {
        InMemoryResponseCacherConfig::builder()
            .ttl(ttl)
            .capacity(2)
            .unwrap()
            .build()
    }

    fn _cacher_options(ttl: Duration) -> InMemoryResponseCacherOptions {
        InMemoryResponseCacherOptions::builder().ttl(ttl).build()
    }

    fn _test_object() -> TestStruct {
        TestStruct {
            field: "test".to_string(),
        }
    }

    #[tokio::test]
    async fn test_put_and_get() -> VcxLedgerResult<()> {
        let cacher = InMemoryResponseCacher::new(_cacher_config(Duration::from_secs(1)));
        let test_object = _test_object();

        cacher.put("id1", test_object.clone()).await?;

        let cached_object: Option<TestStruct> = cacher.get("id1", None).await?;
        assert_eq!(Some(test_object), cached_object);

        Ok(())
    }

    #[tokio::test]
    async fn test_expiration() -> VcxLedgerResult<()> {
        let cacher = InMemoryResponseCacher::new(_cacher_config(Duration::from_millis(1)));
        let test_object = _test_object();

        cacher.put("id1", test_object).await?;

        tokio::time::sleep(Duration::from_millis(1)).await;

        let cached_object: Option<TestStruct> = cacher.get("id1", None).await?;
        assert_eq!(None, cached_object);

        Ok(())
    }

    #[tokio::test]
    async fn test_capacity() -> VcxLedgerResult<()> {
        let cacher = InMemoryResponseCacher::new(_cacher_config(Duration::from_secs(1)));
        let test_object = _test_object();

        cacher.put("id1", test_object.clone()).await?;
        cacher.put("id2", test_object.clone()).await?;
        cacher.put("id3", test_object).await?;

        let cached_object: Option<TestStruct> = cacher.get("id1", None).await?;
        assert_eq!(None, cached_object);

        Ok(())
    }

    #[tokio::test]
    async fn test_nonexistent_key() -> VcxLedgerResult<()> {
        let cacher = InMemoryResponseCacher::new(_cacher_config(Duration::from_secs(1)));

        let cached_object: Option<TestStruct> = cacher.get("nonexistent", None).await?;
        assert_eq!(None, cached_object);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_options_ttl_override_global_config_ttl() -> VcxLedgerResult<()> {
        let cacher = InMemoryResponseCacher::new(_cacher_config(Duration::from_millis(1)));
        let test_object = _test_object();

        cacher.put("id1", test_object.clone()).await?;

        tokio::time::sleep(Duration::from_millis(1)).await;

        let cached_object: Option<TestStruct> = cacher
            .get("id1", Some(_cacher_options(Duration::from_millis(10))))
            .await?;
        assert_eq!(Some(test_object), cached_object);

        Ok(())
    }
}
