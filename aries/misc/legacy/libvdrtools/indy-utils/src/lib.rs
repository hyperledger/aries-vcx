// allow all clippy warnings, given this is legacy to be removed soon
#![allow(clippy::all)]
#[macro_use]
extern crate serde_json;

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
pub mod crypto;
pub mod environment;
pub mod sequence;
pub mod wql;

use indy_api_types::{CommandHandle, SearchHandle, VdrHandle, WalletHandle};

pub fn next_wallet_handle() -> WalletHandle {
    WalletHandle(sequence::get_next_id())
}

pub fn next_command_handle() -> CommandHandle {
    sequence::get_next_id()
}

pub fn next_search_handle() -> SearchHandle {
    SearchHandle(sequence::get_next_id())
}

pub fn next_vdr_handle() -> VdrHandle {
    sequence::get_next_id()
}
