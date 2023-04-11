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
#[cfg(feature = "vdrtools")]
pub mod indy;
pub mod ledger;
pub mod utils;
pub mod wallet;

#[cfg(feature = "vdrtools")]
pub use indy::{
    PoolHandle, SearchHandle, WalletHandle, INVALID_POOL_HANDLE, INVALID_SEARCH_HANDLE, INVALID_WALLET_HANDLE,
};
