use crate::{
    cache::{
        lru::LruCache,
        cache::Cache,
    },
    storage::{
        Tag::{Encrypted, PlainText},
        TagName::{OfEncrypted, OfPlain},
        StorageRecord, Tag, TagName,
    },
    wallet::EncryptedValue,
    RecordOptions,
};
use std::{
    collections::{HashSet, HashMap},
    iter::FromIterator,
    sync::atomic::{AtomicUsize, Ordering},
};
use indy_api_types::domain::wallet::{CacheConfig, CachingAlgorithm};
use async_std::sync::{RwLock, Mutex};

#[derive(PartialEq, Eq, Hash)]
pub struct WalletCacheKey {
    type_: Vec<u8>,
    id: Vec<u8>,
}

pub struct WalletCacheValue {
    value: EncryptedValue,
    tags: Vec<Tag>,
}

pub struct WalletCache {
    cache: Option<Mutex<Box<dyn Cache + Send>>>,
    cache_entities: HashSet<String>,
}

impl WalletCache {
    pub fn new(config: Option<CacheConfig>) -> Self {
        match config {
            Some(cache_config) if cache_config.size > 0 && !cache_config.entities.is_empty() => {
                let cache = match cache_config.algorithm {
                    CachingAlgorithm::LRU => LruCache::new(cache_config.size),
                };
                WalletCache {
                    cache: Some(Mutex::new(Box::new(cache))),
                    cache_entities: HashSet::from_iter(cache_config.entities.iter().cloned()),
                }
            }
            _ => {
                WalletCache { // no cache
                    cache: None,
                    cache_entities: HashSet::new(),
                }
            }
        }
    }

    pub fn is_type_cacheable(&self, type_: &str) -> bool {
        self.cache.is_some() && self.cache_entities.contains(&type_.to_owned())
    }

    pub async fn add(
        &self,
        type_: &str,
        etype: &[u8],
        eid: &[u8],
        evalue: &EncryptedValue,
        etags: &[Tag],
    ) {
        if let Some(protected_cache) = &self.cache {
            if self.cache_entities.contains(&type_.to_owned()) {
                let key = WalletCacheKey {
                    type_: etype.to_owned(),
                    id: eid.to_owned(),
                };
                let value = WalletCacheValue {
                    value: evalue.to_owned(),
                    tags: etags.to_owned(),
                };
                let _ = protected_cache.lock().await.put(key, value);
            }
        }
    }

    pub async fn add_tags(
        &self,
        type_: &str,
        etype: &[u8],
        eid: &[u8],
        etags: &[Tag],
    ) {
        if let Some(protected_cache) = &self.cache {
            if self.cache_entities.contains(&type_.to_owned()) {
                let key = WalletCacheKey {
                    type_: etype.to_owned(),
                    id: eid.to_owned(),
                };
                let _ = protected_cache.lock().await.get_mut(&key).map(|v|{
                    v.tags.append(&mut etags.to_owned())
                });
            }
        }
    }

    pub async fn update_tags(
        &self,
        type_: &str,
        etype: &[u8],
        eid: &[u8],
        etags: &[Tag],
    ) {
        if let Some(protected_cache) = &self.cache {
            if self.cache_entities.contains(&type_.to_owned()) {
                let key = WalletCacheKey {
                    type_: etype.to_owned(),
                    id: eid.to_owned(),
                };
                let _ = protected_cache.lock().await.get_mut(&key).map(|v|{
                    v.tags = etags.to_vec()
                });
            }
        }
    }

    pub async fn delete_tags(
        &self,
        type_: &str,
        etype: &[u8],
        eid: &[u8],
        etag_names: &[TagName],
    ) {
        if let Some(protected_cache) = &self.cache {
            if self.cache_entities.contains(&type_.to_owned()) {
                let key = WalletCacheKey {
                    type_: etype.to_owned(),
                    id: eid.to_owned(),
                };
                let mut enc_tag_names = HashSet::new();
                let mut plain_tag_names = HashSet::new();
                for x in etag_names {
                    match x {
                        OfEncrypted(value) => enc_tag_names.insert(value),
                        OfPlain(value) => plain_tag_names.insert(value),
                    };
                }
                let _ = protected_cache.lock().await.get_mut(&key).map(|v|{
                    v.tags.retain(|el| {
                        match el {
                            Encrypted(tag_name, _) => {
                                !enc_tag_names.contains(tag_name)
                            },
                            PlainText(tag_name, _) => {
                                !plain_tag_names.contains(tag_name)
                            }
                        }
                    });
                });
            }
        }
    }

    pub async fn update(
        &self,
        type_: &str,
        etype: &[u8],
        eid: &[u8],
        evalue: &EncryptedValue,
    ) {
        if let Some(protected_cache) = &self.cache {
            if self.cache_entities.contains(&type_.to_owned()) {
                let key = WalletCacheKey {
                    type_: etype.to_owned(),
                    id: eid.to_owned(),
                };
                let _ = protected_cache.lock().await.get_mut(&key).map(|v|{
                    v.value = evalue.to_owned()
                });
            }
        }
    }

    pub async fn get(
        &self,
        type_: &str,
        etype: &[u8],
        eid: &[u8],
        options: &RecordOptions
    ) -> Option<StorageRecord> {
        if let Some(protected_cache) = &self.cache {
            if self.cache_entities.contains(&type_.to_owned()) {
                let key = WalletCacheKey {
                    type_: etype.to_owned(),
                    id: eid.to_owned(),
                };
                protected_cache.lock().await.get(&key).map(|v|{
                    StorageRecord {
                        id: eid.to_owned(),
                        value: if options.retrieve_value {Some(v.value.clone())} else {None},
                        type_: if options.retrieve_type {Some(etype.to_owned())} else {None},
                        tags: if options.retrieve_tags {Some(v.tags.clone())} else {None},
                    }
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub async fn delete(&self, type_: &str, etype: &[u8], eid: &[u8]) {
        if let Some(protected_cache) = &self.cache {
            if self.cache_entities.contains(&type_.to_owned()) {
                let key = WalletCacheKey {
                    type_: etype.to_owned(),
                    id: eid.to_owned(),
                };
                let _ = protected_cache.lock().await.pop(&key);
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct WalletCacheHitData {
    pub hit: AtomicUsize,
    pub miss: AtomicUsize,
    pub not_cached: AtomicUsize,
}

impl WalletCacheHitData {
    fn inc(var: &AtomicUsize, increment: usize) -> usize {
        var.fetch_add(increment, Ordering::Relaxed)
    }

    fn get(var: &AtomicUsize) -> usize {
        var.load(Ordering::Relaxed)
    }

    pub fn inc_hit(&self) -> usize {
        WalletCacheHitData::inc(&self.hit, 1)
    }

    pub fn inc_miss(&self) -> usize {
        WalletCacheHitData::inc(&self.miss, 1)
    }

    pub fn inc_not_cached(&self) -> usize {
        WalletCacheHitData::inc(&self.not_cached, 1)
    }

    pub fn get_hit(&self) -> usize {
        WalletCacheHitData::get(&self.hit)
    }

    pub fn get_miss(&self) -> usize {
        WalletCacheHitData::get(&self.miss)
    }

    pub fn get_not_cached(&self) -> usize {
        WalletCacheHitData::get(&self.not_cached)
    }
}

impl Clone for WalletCacheHitData {
    fn clone(&self) -> Self {
        WalletCacheHitData {
            hit: AtomicUsize::from(self.get_hit()),
            miss: AtomicUsize::from(self.get_miss()),
            not_cached: AtomicUsize::from(self.get_not_cached())
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self.hit.get_mut() = source.get_hit();
        *self.miss.get_mut() = source.get_miss();
        *self.not_cached.get_mut() = source.get_not_cached();
    }
}

pub struct WalletCacheHitMetrics {
    pub data: RwLock<HashMap<String, WalletCacheHitData>>,
}

impl WalletCacheHitMetrics {
    pub fn new() -> Self {
        WalletCacheHitMetrics {
            data: RwLock::new(HashMap::new())
        }
    }

    pub async fn inc_cache_hit(&self, type_: &str) -> usize {
        self.update_data(type_, |x| x.inc_hit()).await
    }

    pub async fn inc_cache_miss(&self, type_: &str) -> usize {
        self.update_data(type_, |x| x.inc_miss()).await
    }

    pub async fn inc_not_cached(&self, type_: &str) -> usize {
        self.update_data(type_, |x| x.inc_not_cached()).await
    }

    async fn update_data(&self, type_: &str, f: fn(&WalletCacheHitData) -> usize) -> usize {
        let read_guard = self.data.read().await;
        match read_guard.get(type_) {
            Some(x) => f(x),
            None => {
                drop(read_guard);
                let mut write_guard = self.data.write().await;
                // check if data is inserted in the mean time until write lock is acquired.
                match write_guard.get(type_) {
                    Some(x) => f(x),
                    None => {
                        // we are now holding exclusive access, so insert the item in map.
                        let d = Default::default();
                        let result = f(&d);
                        write_guard.insert(type_.to_string(), d);
                        result
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
    pub async fn get_data_for_type(&self, type_: &str) -> Option<WalletCacheHitData> {
        self.data.read().await.get(type_).map(|x|x.clone())
    }

    pub async fn get_data(&self) -> HashMap<String, WalletCacheHitData> {
        self.data.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;

    use super::*;
    use crate::storage::{Tag, TagName};
    use rand::{distributions::Uniform, distributions::Alphanumeric, Rng};
    use indy_api_types::domain::wallet::DEFAULT_CACHE_SIZE;
    use futures::Future;
    use std::time::Duration;

    const TYPE_A: &str = "TypeA";
    const TYPE_B: &str = "TypeB";
    const TYPE_NON_CACHED: &str = "TypeNonCached";

    const ETYPE1: &[u8] = &[1, 2, 3, 1];
    const EID1: &[u8] = &[2, 3, 4, 1];
    const EID2: &[u8] = &[2, 3, 4, 2];

    const FULL_OPTIONS: RecordOptions = RecordOptions {
        retrieve_type: true,
        retrieve_value: true,
        retrieve_tags: true
    };

    fn _rand_vec(size: usize) -> Vec<u8> {
        rand::thread_rng().sample_iter(&Uniform::new(0, 255)).take(size).collect()
    }

    fn _rand_str(size: usize) -> String {
        rand::thread_rng().sample_iter(&Alphanumeric).take(size).map(char::from).collect()
    }

    fn _enc_value() -> EncryptedValue {
        EncryptedValue {
            data: _rand_vec(200),
            key: _rand_vec(20)
        }
    }

    fn _enc_tag() -> Tag {
        if rand::thread_rng().gen::<u8>() % 2 == 1 {
            Tag::Encrypted(_rand_vec(20), _rand_vec(100))
        } else {
            Tag::PlainText(_rand_vec(20), _rand_str(100))
        }
    }

    fn _cache() -> WalletCache {
        let config = CacheConfig {
            size: 10,
            entities: vec![TYPE_A.to_string(), TYPE_B.to_string()],
            algorithm: CachingAlgorithm::LRU
        };
        WalletCache::new(Some(config))
    }

    fn _no_cache() -> WalletCache {
        let config = CacheConfig {
            size: 10,
            entities: vec![],
            algorithm: CachingAlgorithm::LRU
        };
        WalletCache::new(Some(config))
    }

    fn _vec_to_hash_set(items: &[&str]) -> HashSet<String> {
        HashSet::from_iter(items.into_iter().map(|el|el.to_string()))
    }

    fn _tag_names(tags: &[Tag]) -> Vec<TagName> {
        tags.into_iter().map(|el|{
            match el {
                Encrypted(key, _) => TagName::OfEncrypted(key.to_owned()),
                PlainText(key, _) => TagName::OfPlain(key.to_owned()),
            }
        }).collect()
    }

    #[test]
    fn new_with_no_config_works() {
        let cache = WalletCache::new(None);
        assert!(cache.cache.is_none());
        assert_eq!(cache.cache_entities.len(), 0);
    }

    #[test]
    fn new_with_default_config_works() {
        let config = CacheConfig {
            size: DEFAULT_CACHE_SIZE,
            entities: vec![],
            algorithm: CachingAlgorithm::LRU
        };
        let cache = WalletCache::new(Some(config));
        assert!(cache.cache.is_none());
        assert_eq!(cache.cache_entities.len(), 0);
    }

    #[test]
    fn new_with_size_but_no_entities_in_config_works() {
        let config = CacheConfig {
            size: 20,
            entities: vec![],
            algorithm: CachingAlgorithm::LRU
        };
        let cache = WalletCache::new(Some(config));
        assert!(cache.cache.is_none());
        assert_eq!(cache.cache_entities.len(), 0);
    }

    #[test]
    fn new_with_default_size_in_config_works() {
        let config_str = json!({
            "entities": vec![TYPE_A.to_string(), TYPE_B.to_string()]
        }).to_string();
        let config: CacheConfig = serde_json::from_str(&config_str).unwrap();
        let wallet_cache = WalletCache::new(Some(config));
        assert!(wallet_cache.cache.is_some());
        let mut cache = wallet_cache.cache.unwrap();
        assert_eq!(cache.get_mut().cap(), DEFAULT_CACHE_SIZE);
        assert_eq!(cache.get_mut().len(), 0);
        assert_eq!(wallet_cache.cache_entities.len(), 2);
        assert_eq!(wallet_cache.cache_entities, _vec_to_hash_set(&[TYPE_A, TYPE_B]));
    }

    #[test]
    fn new_with_size_in_config_works() {
        let config = CacheConfig {
            size: 20,
            entities: vec![TYPE_A.to_string(), TYPE_B.to_string()],
            algorithm: CachingAlgorithm::LRU
        };
        let wallet_cache = WalletCache::new(Some(config));
        assert!(wallet_cache.cache.is_some());
        let mut cache = wallet_cache.cache.unwrap();
        assert_eq!(cache.get_mut().cap(), 20);
        assert_eq!(cache.get_mut().len(), 0);
        assert_eq!(wallet_cache.cache_entities.len(), 2);
        assert_eq!(wallet_cache.cache_entities, _vec_to_hash_set(&[TYPE_A, TYPE_B]));
    }

    #[test]
    fn is_type_cacheable_works() {
        let cache = _cache();
        let result = cache.is_type_cacheable(TYPE_A);
        assert_eq!(result, true);
    }

    #[test]
    fn is_type_cacheable_for_noncacheable_type_works() {
        let cache = _cache();
        let result = cache.is_type_cacheable(TYPE_NON_CACHED);
        assert_eq!(result, false);
    }

    #[test]
    fn is_type_cacheable_for_no_cache_enabled_works() {
        let cache = _no_cache();
        let result = cache.is_type_cacheable(TYPE_A);
        assert_eq!(result, false);
    }

    #[async_std::test]
    async fn add_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1.clone(), tag2.clone()]).await;

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value);
        assert_eq!(cached.tags, vec![tag1, tag2]);
    }

    #[async_std::test]
    async fn add_without_tags_works() {
        let value = _enc_value();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[]).await;

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value);
        assert_eq!(cached.tags, vec![]);
    }

    #[async_std::test]
    async fn add_for_non_cacheable_type_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_NON_CACHED, ETYPE1, EID1, &value, &[tag1, tag2]).await;

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 0);
    }

    #[async_std::test]
    async fn add_for_no_cache_enabled_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _no_cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1, tag2]).await;
        assert!(cache.cache.is_none());
    }

    #[async_std::test]
    async fn add_tags_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();
        let tag3 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1.clone(), tag2.clone()]).await;
        cache.add_tags(TYPE_A, ETYPE1, EID1, &[tag3.clone()]).await;

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value);
        assert_eq!(cached.tags, vec![tag1, tag2, tag3]);
    }

    #[async_std::test]
    async fn add_tags_on_item_without_tags_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[]).await;
        cache.add_tags(TYPE_A, ETYPE1, EID1, &[tag1.clone(), tag2.clone()]).await;

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value);
        assert_eq!(cached.tags, vec![tag1, tag2]);
    }

    #[async_std::test]
    async fn add_tags_on_non_cached_item_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();
        let tag3 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1.clone(), tag2.clone()]).await;
        cache.add_tags(TYPE_A, ETYPE1, EID2, &[tag3]).await;

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value);
        assert_eq!(cached.tags, vec![tag1, tag2]);

        let key2 = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID2.to_vec()
        };

        assert!(lru.peek(&key2).is_none());
    }

    #[async_std::test]
    async fn add_tags_for_non_cacheable_type_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();
        let tag3 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_NON_CACHED, ETYPE1, EID1, &value, &[tag1, tag2]).await;
        cache.add_tags(TYPE_NON_CACHED, ETYPE1, EID1, &[tag3]).await;

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 0);
    }

    #[async_std::test]
    async fn add_tags_for_no_cache_enabled_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();
        let tag3 = _enc_tag();

        let cache = _no_cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1, tag2]).await;
        cache.add_tags(TYPE_A, ETYPE1, EID1, &[tag3]).await;

        assert!(cache.cache.is_none());
    }

    #[async_std::test]
    async fn update_tags_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();
        let tag3 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1, tag2]).await;
        cache.update_tags(TYPE_A, ETYPE1, EID1, &[tag3.clone()]).await;

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value);
        assert_eq!(cached.tags, vec![tag3]);
    }

    #[async_std::test]
    async fn update_tags_on_item_without_tags_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[]).await;
        cache.update_tags(TYPE_A, ETYPE1, EID1, &[tag1.clone()]).await;

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value);
        assert_eq!(cached.tags, vec![tag1]);
    }

    #[async_std::test]
    async fn update_tags_on_non_cached_item_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();
        let tag3 = _enc_tag();
        let tag4 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1.clone(), tag2.clone()]).await;
        cache.update_tags(TYPE_A, ETYPE1, EID2, &[tag3, tag4]).await;

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value);
        assert_eq!(cached.tags, vec![tag1, tag2]);

        let key2 = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID2.to_vec()
        };

        assert!(lru.peek(&key2).is_none());
    }

    #[async_std::test]
    async fn update_tags_for_non_cacheable_type_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();
        let tag3 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_NON_CACHED, ETYPE1, EID1, &value, &[tag1, tag2]).await;
        cache.update_tags(TYPE_NON_CACHED, ETYPE1, EID1, &[tag3]).await;

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 0);
    }

    #[async_std::test]
    async fn update_tags_for_no_cache_enabled_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();
        let tag3 = _enc_tag();

        let cache = _no_cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1, tag2]).await;
        cache.update_tags(TYPE_A, ETYPE1, EID1, &[tag3]).await;

        assert!(cache.cache.is_none());
    }

    #[async_std::test]
    async fn delete_tags_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();
        let tag3 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1.clone(), tag2.clone()]).await;
        cache.delete_tags(TYPE_A, ETYPE1, EID1, &_tag_names(&[tag1, tag3])).await;

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value);
        assert_eq!(cached.tags, vec![tag2]);
    }

    #[async_std::test]
    async fn delete_tags_on_item_without_tags_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[]).await;
        cache.delete_tags(TYPE_A, ETYPE1, EID1, &_tag_names(&[tag1])).await;

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value);
        assert_eq!(cached.tags, vec![]);
    }

    #[async_std::test]
    async fn delete_tags_on_non_cached_item_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1.clone(), tag2.clone()]).await;
        cache.delete_tags(TYPE_A, ETYPE1, EID2, &_tag_names(&[tag1.clone()])).await;

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value);
        assert_eq!(cached.tags, vec![tag1, tag2]);

        let key2 = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID2.to_vec()
        };

        assert!(lru.peek(&key2).is_none());
    }

    #[async_std::test]
    async fn delete_tags_for_non_cacheable_type_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_NON_CACHED, ETYPE1, EID1, &value, &[tag1.clone(), tag2.clone()]).await;
        cache.delete_tags(TYPE_NON_CACHED, ETYPE1, EID1, &_tag_names(&[tag1.clone()])).await;

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 0);
    }

    #[async_std::test]
    async fn delete_tags_for_no_cache_enabled_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _no_cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1.clone(), tag2.clone()]).await;
        cache.delete_tags(TYPE_A, ETYPE1, EID1, &_tag_names(&[tag1])).await;

        assert!(cache.cache.is_none());
    }

    #[async_std::test]
    async fn update_works() {
        let value = _enc_value();
        let value2 = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1.clone(), tag2.clone()]).await;
        cache.update(TYPE_A, ETYPE1, EID1, &value2).await;

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value2);
        assert_eq!(cached.tags, vec![tag1, tag2]);
    }

    #[async_std::test]
    async fn update_on_item_without_tags_works() {
        let value = _enc_value();
        let value2 = _enc_value();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[]).await;
        cache.update(TYPE_A, ETYPE1, EID1, &value2).await;

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value2);
        assert_eq!(cached.tags, vec![]);
    }

    #[async_std::test]
    async fn update_on_non_cached_item_works() {
        let value = _enc_value();
        let value2 = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1.clone(), tag2.clone()]).await;
        cache.update(TYPE_A, ETYPE1, EID2, &value2).await;

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value);
        assert_eq!(cached.tags, vec![tag1, tag2]);

        let key2 = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID2.to_vec()
        };

        assert!(lru.peek(&key2).is_none());
    }

    #[async_std::test]
    async fn update_for_non_cacheable_type_works() {
        let value = _enc_value();
        let value2 = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_NON_CACHED, ETYPE1, EID1, &value, &[tag1, tag2]).await;
        cache.update(TYPE_NON_CACHED, ETYPE1, EID1, &value2).await;

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 0);
    }

    #[async_std::test]
    async fn update_for_no_cache_enabled_works() {
        let value = _enc_value();
        let value2 = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _no_cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1, tag2]).await;
        cache.update(TYPE_A, ETYPE1, EID1, &value2).await;

        assert!(cache.cache.is_none());
    }

    #[async_std::test]
    async fn delete_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1, tag2]).await;
        cache.delete(TYPE_A, ETYPE1, EID1).await;

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 0);
        assert!(lru.peek(&key).is_none());
    }

    #[async_std::test]
    async fn delete_on_item_without_tags_works() {
        let value = _enc_value();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[]).await;
        cache.delete(TYPE_A, ETYPE1, EID1).await;

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 0);
        assert!(lru.peek(&key).is_none());
    }

    #[async_std::test]
    async fn delete_on_non_cached_item_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1.clone(), tag2.clone()]).await;
        cache.delete(TYPE_A, ETYPE1, EID2).await;

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value);
        assert_eq!(cached.tags, vec![tag1, tag2]);

        let key2 = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID2.to_vec()
        };

        assert!(lru.peek(&key2).is_none());
    }

    #[async_std::test]
    async fn delete_for_non_cacheable_type_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_NON_CACHED, ETYPE1, EID1, &value, &[tag1, tag2]).await;
        cache.delete(TYPE_NON_CACHED, ETYPE1, EID1).await;

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 0);
    }

    #[async_std::test]
    async fn delete_for_no_cache_enabled_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _no_cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1, tag2]).await;
        cache.delete(TYPE_A, ETYPE1, EID1).await;

        assert!(cache.cache.is_none());
    }

    #[async_std::test]
    async fn get_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1.clone(), tag2.clone()]).await;
        let result = cache.get(TYPE_A, ETYPE1, EID1, &FULL_OPTIONS).await.unwrap();

        assert_eq!(result.id, EID1);
        assert_eq!(result.type_, Some(ETYPE1.to_owned()));
        assert_eq!(result.value, Some(value.clone()));
        assert_eq!(result.tags, Some(vec![tag1.clone(), tag2.clone()]));

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value);
        assert_eq!(cached.tags, vec![tag1, tag2]);
    }

    #[async_std::test]
    async fn get_for_item_without_tags_works() {
        let value = _enc_value();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[]).await;
        let result = cache.get(TYPE_A, ETYPE1, EID1, &FULL_OPTIONS).await.unwrap();

        assert_eq!(result.id, EID1);
        assert_eq!(result.type_, Some(ETYPE1.to_owned()));
        assert_eq!(result.value, Some(value.clone()));
        assert_eq!(result.tags, Some(vec![]));

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value);
        assert_eq!(cached.tags, vec![]);
    }

    #[async_std::test]
    async fn get_for_non_cached_item_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1.clone(), tag2.clone()]).await;
        let result = cache.get(TYPE_A, ETYPE1, EID2, &FULL_OPTIONS).await;

        assert!(result.is_none());

        let key = WalletCacheKey {
            type_: ETYPE1.to_vec(),
            id: EID1.to_vec()
        };

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 1);
        let cached = lru.peek(&key).unwrap();
        assert_eq!(cached.value, value);
        assert_eq!(cached.tags, vec![tag1, tag2]);
    }

    #[async_std::test]
    async fn get_for_non_cacheable_type_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _cache();

        cache.add(TYPE_NON_CACHED, ETYPE1, EID1, &value, &[tag1, tag2]).await;
        let result = cache.get(TYPE_A, ETYPE1, EID1, &FULL_OPTIONS).await;

        assert!(result.is_none());

        let mut internal_cache = cache.cache.unwrap();
        let lru = internal_cache.get_mut();
        assert_eq!(lru.len(), 0);
    }

    #[async_std::test]
    async fn get_for_no_cache_enabled_works() {
        let value = _enc_value();
        let tag1 = _enc_tag();
        let tag2 = _enc_tag();

        let cache = _no_cache();

        cache.add(TYPE_A, ETYPE1, EID1, &value, &[tag1, tag2]).await;
        let result = cache.get(TYPE_A, ETYPE1, EID1, &FULL_OPTIONS).await;

        assert!(result.is_none());

        assert!(cache.cache.is_none());
    }

    #[async_std::test]
    async fn wallet_cache_hit_metrics_new_works() {
        let mut metrics = WalletCacheHitMetrics::new();

        assert!(metrics.data.get_mut().is_empty());
    }

    #[async_std::test]
    async fn wallet_cache_hit_metrics_inc_cache_hit_works() {
        let metrics = WalletCacheHitMetrics::new();

        metrics.inc_cache_hit(TYPE_A).await;

        let type_data = metrics.get_data_for_type(TYPE_A).await.unwrap();
        assert_eq!(type_data.get_hit(), 1);
        assert_eq!(type_data.get_miss(), 0);
        assert_eq!(type_data.get_not_cached(), 0);
    }

    #[async_std::test]
    async fn wallet_cache_hit_metrics_inc_cache_miss_works() {
        let metrics = WalletCacheHitMetrics::new();

        metrics.inc_cache_miss(TYPE_A).await;

        let type_data = metrics.get_data_for_type(TYPE_A).await.unwrap();
        assert_eq!(type_data.get_hit(), 0);
        assert_eq!(type_data.get_miss(), 1);
        assert_eq!(type_data.get_not_cached(), 0);
    }

    #[async_std::test]
    async fn wallet_cache_hit_metrics_inc_not_cached_works() {
        let metrics = WalletCacheHitMetrics::new();

        metrics.inc_not_cached(TYPE_A).await;

        let type_data = metrics.get_data_for_type(TYPE_A).await.unwrap();
        assert_eq!(type_data.get_hit(), 0);
        assert_eq!(type_data.get_miss(), 0);
        assert_eq!(type_data.get_not_cached(), 1);
    }

    #[async_std::test]
    async fn wallet_cache_hit_metrics_get_data_works() {
        let metrics = WalletCacheHitMetrics::new();

        let fut1 = metrics.inc_cache_hit(TYPE_A);
        let fut2 = metrics.inc_cache_miss(TYPE_A);
        let fut3 = metrics.inc_cache_miss(TYPE_B);
        let fut4 = metrics.inc_not_cached(TYPE_NON_CACHED);

        let result = futures::future::join4(fut1, fut2, fut3, fut4).await;
        assert_eq!(result, (0, 0, 0, 0));

        let data = metrics.get_data().await;

        assert_eq!(data.len(), 3);
        assert_eq!(data.get(TYPE_A).unwrap().get_hit(), 1);
        assert_eq!(data.get(TYPE_A).unwrap().get_miss(), 1);
        assert_eq!(data.get(TYPE_A).unwrap().get_not_cached(), 0);
        assert_eq!(data.get(TYPE_B).unwrap().get_hit(), 0);
        assert_eq!(data.get(TYPE_B).unwrap().get_miss(), 1);
        assert_eq!(data.get(TYPE_B).unwrap().get_not_cached(), 0);
        assert_eq!(data.get(TYPE_NON_CACHED).unwrap().get_hit(), 0);
        assert_eq!(data.get(TYPE_NON_CACHED).unwrap().get_miss(), 0);
        assert_eq!(data.get(TYPE_NON_CACHED).unwrap().get_not_cached(), 1);
    }

    #[async_std::test]
    async fn wallet_cache_hit_metrics_get_data_for_type_works() {
        let metrics = WalletCacheHitMetrics::new();

        let fut1 = metrics.inc_cache_hit(TYPE_A);
        let fut2 = metrics.inc_cache_miss(TYPE_A);
        let fut3 = metrics.inc_cache_miss(TYPE_B);
        let fut4 = metrics.inc_not_cached(TYPE_NON_CACHED);

        let result = futures::future::join4(fut1, fut2, fut3, fut4).await;
        assert_eq!(result, (0, 0, 0, 0));

        let data_a = metrics.get_data_for_type(TYPE_A).await.unwrap();
        let data_b = metrics.get_data_for_type(TYPE_B).await.unwrap();
        let data_nc = metrics.get_data_for_type(TYPE_NON_CACHED).await.unwrap();

        assert_eq!(data_a.get_hit(), 1);
        assert_eq!(data_a.get_miss(), 1);
        assert_eq!(data_a.get_not_cached(), 0);
        assert_eq!(data_b.get_hit(), 0);
        assert_eq!(data_b.get_miss(), 1);
        assert_eq!(data_b.get_not_cached(), 0);
        assert_eq!(data_nc.get_hit(), 0);
        assert_eq!(data_nc.get_miss(), 0);
        assert_eq!(data_nc.get_not_cached(), 1);
    }

    #[async_std::test]
    async fn wallet_cache_hit_metrics_get_data_works_with_empty() {
        let metrics = WalletCacheHitMetrics::new();

        assert!(metrics.get_data().await.is_empty());
    }

    #[async_std::test]
    async fn wallet_cache_hit_metrics_get_data_for_type_works_with_empty() {
        let metrics = WalletCacheHitMetrics::new();

        assert!(metrics.get_data_for_type(TYPE_A).await.is_none());
    }

    async fn _execute_with_random_delay<F>(future: F) -> usize
        where F: Future<Output=usize>
    {
        async_std::task::sleep(Duration::from_millis(rand::thread_rng().gen_range(0, 1000))).await;
        future.await + 0
    }

    #[async_std::test]
    async fn wallet_cache_hit_metrics_work_correctly_under_concurrent_load() {
        let metrics = WalletCacheHitMetrics::new();
        let mut futures1 = vec![];
        let mut futures2 = vec![];
        let mut futures3 = vec![];

        for _ in 0..1000 {
            futures1.push(_execute_with_random_delay(metrics.inc_cache_hit(TYPE_A)));
            futures2.push(_execute_with_random_delay(metrics.inc_cache_miss(TYPE_A)));
            futures3.push(_execute_with_random_delay(metrics.inc_not_cached(TYPE_NON_CACHED)));
        }

        let result = futures::future::join3(
            futures::future::join_all(futures1),
            futures::future::join_all(futures2),
            futures::future::join_all(futures3)
        ).await;
        println!("result: {:?}", result);

        let type_a_data = metrics.get_data_for_type(TYPE_A).await.unwrap();
        assert!(metrics.get_data_for_type(TYPE_B).await.is_none());
        let type_b_data = metrics.get_data_for_type(TYPE_NON_CACHED).await.unwrap();

        assert_eq!(type_a_data.get_hit(), 1000);
        assert_eq!(type_a_data.get_miss(), 1000);
        assert_eq!(type_a_data.get_not_cached(), 0);
        assert_eq!(type_b_data.get_hit(), 0);
        assert_eq!(type_b_data.get_miss(), 0);
        assert_eq!(type_b_data.get_not_cached(), 1000);
    }
}
