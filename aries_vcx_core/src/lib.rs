#[cfg(debug_assertions)]
#[macro_export]
macro_rules! secret {
    ($val:expr) => {{
        $val
    }};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! secret {
    ($val:expr) => {{
        "_"
    }};
}

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate log;

#[macro_use]
extern crate derive_builder;

pub mod anoncreds;
mod common;
pub mod errors;
pub mod global;
pub mod indy;
pub mod ledger;
pub mod utils;
pub mod wallet;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct WalletHandle(pub vdrtools::WalletHandle);
pub const INVALID_WALLET_HANDLE: WalletHandle = WalletHandle(vdrtools::INVALID_WALLET_HANDLE);

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct SearchHandle(pub vdrtools::SearchHandle);
pub const INVALID_SEARCH_HANDLE: SearchHandle = SearchHandle(vdrtools::INVALID_SEARCH_HANDLE);

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct PoolHandle(pub vdrtools::PoolHandle);
pub const INVALID_POOL_HANDLE: PoolHandle = PoolHandle(vdrtools::INVALID_POOL_HANDLE);
