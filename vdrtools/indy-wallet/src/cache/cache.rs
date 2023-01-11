use crate::cache::wallet_cache::{WalletCacheKey, WalletCacheValue};

pub trait Cache {
    fn put(&mut self, key: WalletCacheKey, value: WalletCacheValue) -> Option<WalletCacheValue> ;
    fn get(&mut self, key: &WalletCacheKey) -> Option<&WalletCacheValue>;
    fn get_mut(&mut self, key: &WalletCacheKey) -> Option<&mut WalletCacheValue>;
    fn pop(&mut self, key: &WalletCacheKey) -> Option<WalletCacheValue>;
    fn peek(&self, key: &WalletCacheKey) -> Option<&WalletCacheValue>;
    fn len(&self) -> usize;
    fn cap(&self) -> usize;
}