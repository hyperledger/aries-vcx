#[cfg(debug_assertions)]
#[macro_export]
macro_rules! secret {
    ($val:expr) => {{
        $val
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

pub extern crate vdrtools; // TODO: REMOVE THIS!

pub mod anoncreds;
mod common;
pub mod errors;
mod global;
pub mod indy;
pub mod ledger;
pub mod utils;
pub mod wallet;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct WalletHandle(pub vdrtools::WalletHandle);
pub const INVALID_WALLET_HANDLE: WalletHandle = WalletHandle(vdrtools::INVALID_WALLET_HANDLE);
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct PoolHandle(pub vdrtools::PoolHandle);
pub const INVALID_POOL_HANDLE: PoolHandle = PoolHandle(vdrtools::INVALID_POOL_HANDLE);
