mod lru;
pub mod wallet_cache;

use crate::cache::wallet_cache::{WalletCacheKey, WalletCacheValue};

pub trait Cache {
    fn put(&mut self, key: WalletCacheKey, value: WalletCacheValue) -> Option<WalletCacheValue>;
    fn get(&mut self, key: &WalletCacheKey) -> Option<&WalletCacheValue>;
    fn get_mut(&mut self, key: &WalletCacheKey) -> Option<&mut WalletCacheValue>;
    fn pop(&mut self, key: &WalletCacheKey) -> Option<WalletCacheValue>;
    #[allow(dead_code)]
    fn peek(&self, key: &WalletCacheKey) -> Option<&WalletCacheValue>;
    #[allow(dead_code)]
    fn len(&self) -> usize;
    #[allow(dead_code)]
    fn cap(&self) -> usize;
}
