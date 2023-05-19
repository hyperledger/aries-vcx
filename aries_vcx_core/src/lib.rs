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
pub use vdrtools::{
    PoolHandle, SearchHandle, WalletHandle, INVALID_POOL_HANDLE, INVALID_SEARCH_HANDLE, INVALID_WALLET_HANDLE,
};

#[cfg(feature = "vdr_proxy_ledger")]
pub use indy_vdr_proxy_client::VdrProxyClient;

#[cfg(any(feature = "modular_libs", feature = "vdr_proxy_ledger"))]
pub use indy_ledger_response_parser::ResponseParser;
