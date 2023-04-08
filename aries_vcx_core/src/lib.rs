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

pub mod anoncreds;
mod common;
mod errors;
mod global;
pub(crate) mod indy;
pub mod ledger;
mod utils;
pub mod wallet;
