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

extern crate serde_json;

#[macro_use]
extern crate log;

pub mod anoncreds;
pub mod errors;
pub mod global;
pub mod ledger;
pub mod utils;
pub mod wallet;

pub use indy_ledger_response_parser::ResponseParser;
pub use indy_vdr::config::PoolConfig;
#[cfg(feature = "vdr_proxy_ledger")]
pub use indy_vdr_proxy_client::VdrProxyClient;
#[cfg(feature = "vdrtools_wallet")]
pub use vdrtools::{SearchHandle, WalletHandle, INVALID_SEARCH_HANDLE, INVALID_WALLET_HANDLE};
