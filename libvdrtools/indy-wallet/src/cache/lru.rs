use crate::cache::cache::Cache;
use crate::cache::wallet_cache::{WalletCacheKey, WalletCacheValue};
use lru::{LruCache as InnerCache};

pub struct LruCache {
    inner: InnerCache<WalletCacheKey, WalletCacheValue>
}

impl LruCache {
    pub fn new(size: usize) -> LruCache {
        LruCache {
            inner: InnerCache::new(size)
        }
    }
}

impl Cache for LruCache {
    fn put(&mut self, key: WalletCacheKey, value: WalletCacheValue) -> Option<WalletCacheValue> {
        self.inner.put(key, value)
    }
    
    fn get(&mut self, key: &WalletCacheKey) -> Option<&WalletCacheValue> {
        self.inner.get(key)
    }
    
    fn get_mut(&mut self, key: &WalletCacheKey) -> Option<&mut WalletCacheValue> {
        self.inner.get_mut(key)
    }
    
    fn pop(&mut self, key: &WalletCacheKey) -> Option<WalletCacheValue> {
        self.inner.pop(key)
    }

    fn peek(&self, key: &WalletCacheKey) -> Option<&WalletCacheValue> {
        self.inner.peek(key)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn cap(&self) -> usize {
        self.inner.cap()
    }
}

