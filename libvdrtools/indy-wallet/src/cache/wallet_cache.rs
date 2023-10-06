use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex, RwLock,
    },
};

use indy_api_types::domain::wallet::{CacheConfig, CachingAlgorithm};

use crate::{
    cache::{lru::LruCache, Cache},
    storage::{
        StorageRecord, Tag,
        Tag::{Encrypted, PlainText},
        TagName,
        TagName::{OfEncrypted, OfPlain},
    },
    wallet::EncryptedValue,
    RecordOptions,
};

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
                    CachingAlgorithm::LRU => {
                        LruCache::new(NonZeroUsize::new(cache_config.size).unwrap())
                    }
                };
                WalletCache {
                    cache: Some(Mutex::new(Box::new(cache))),
                    cache_entities: HashSet::from_iter(cache_config.entities.iter().cloned()),
                }
            }
            _ => {
                WalletCache {
                    // no cache
                    cache: None,
                    cache_entities: HashSet::new(),
                }
            }
        }
    }

    pub fn is_type_cacheable(&self, type_: &str) -> bool {
        self.cache.is_some() && self.cache_entities.contains(&type_.to_owned())
    }

    pub fn add(
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
                let _ = protected_cache.lock().unwrap().put(key, value);
            }
        }
    }

    pub async fn add_tags(&self, type_: &str, etype: &[u8], eid: &[u8], etags: &[Tag]) {
        if let Some(protected_cache) = &self.cache {
            if self.cache_entities.contains(&type_.to_owned()) {
                let key = WalletCacheKey {
                    type_: etype.to_owned(),
                    id: eid.to_owned(),
                };
                let _ = protected_cache
                    .lock()
                    .unwrap() //await
                    .get_mut(&key)
                    .map(|v| v.tags.append(&mut etags.to_owned()));
            }
        }
    }

    pub async fn update_tags(&self, type_: &str, etype: &[u8], eid: &[u8], etags: &[Tag]) {
        if let Some(protected_cache) = &self.cache {
            if self.cache_entities.contains(&type_.to_owned()) {
                let key = WalletCacheKey {
                    type_: etype.to_owned(),
                    id: eid.to_owned(),
                };
                let _ = protected_cache
                    .lock()
                    .unwrap() //await
                    .get_mut(&key)
                    .map(|v| v.tags = etags.to_vec());
            }
        }
    }

    pub async fn delete_tags(&self, type_: &str, etype: &[u8], eid: &[u8], etag_names: &[TagName]) {
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
                let _ = protected_cache
                    .lock()
                    .unwrap() //await
                    .get_mut(&key)
                    .map(|v| {
                        v.tags.retain(|el| match el {
                            Encrypted(tag_name, _) => !enc_tag_names.contains(tag_name),
                            PlainText(tag_name, _) => !plain_tag_names.contains(tag_name),
                        });
                    });
            }
        }
    }

    pub async fn update(&self, type_: &str, etype: &[u8], eid: &[u8], evalue: &EncryptedValue) {
        if let Some(protected_cache) = &self.cache {
            if self.cache_entities.contains(&type_.to_owned()) {
                let key = WalletCacheKey {
                    type_: etype.to_owned(),
                    id: eid.to_owned(),
                };
                let _ = protected_cache
                    .lock()
                    .unwrap() // await
                    .get_mut(&key)
                    .map(|v| v.value = evalue.to_owned());
            }
        }
    }

    pub async fn get(
        &self,
        type_: &str,
        etype: &[u8],
        eid: &[u8],
        options: &RecordOptions,
    ) -> Option<StorageRecord> {
        if let Some(protected_cache) = &self.cache {
            if self.cache_entities.contains(&type_.to_owned()) {
                let key = WalletCacheKey {
                    type_: etype.to_owned(),
                    id: eid.to_owned(),
                };
                protected_cache
                    .lock()
                    .unwrap() //await
                    .get(&key)
                    .map(|v| StorageRecord {
                        id: eid.to_owned(),
                        value: if options.retrieve_value {
                            Some(v.value.clone())
                        } else {
                            None
                        },
                        type_: if options.retrieve_type {
                            Some(etype.to_owned())
                        } else {
                            None
                        },
                        tags: if options.retrieve_tags {
                            Some(v.tags.clone())
                        } else {
                            None
                        },
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
                let _ = protected_cache
                    .lock()
                    .unwrap() //await
                    .pop(&key);
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
            not_cached: AtomicUsize::from(self.get_not_cached()),
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
            data: RwLock::new(HashMap::new()),
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
        let read_guard = self.data.read().unwrap(); //await;
        match read_guard.get(type_) {
            Some(x) => f(x),
            None => {
                drop(read_guard);
                let mut write_guard = self.data.write().unwrap(); //await;
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
        self.data.read().unwrap().get(type_).cloned()
    }

    pub fn get_data(&self) -> HashMap<String, WalletCacheHitData> {
        self.data.read().unwrap().clone()
    }
}
