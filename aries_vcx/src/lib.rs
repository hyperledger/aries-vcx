#![allow(clippy::or_fun_call)]
#![allow(clippy::module_inception)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::new_without_default)]
#![allow(clippy::inherent_to_string)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::diverging_sub_expression)]
#![deny(clippy::unwrap_used)]
#![cfg_attr(feature = "fatal_warnings", deny(warnings))]
#![crate_name = "aries_vcx"]
//this is needed for some large json macro invocations
#![recursion_limit = "128"]

pub extern crate agency_client;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate derive_builder;

#[cfg(test)]
extern crate async_channel;

pub extern crate aries_vcx_core;
pub extern crate messages;

#[macro_use]
pub mod utils;

#[macro_use]
pub mod handlers;

pub mod global;
pub mod protocols;

pub mod common;
pub mod core;
pub mod errors;
pub mod transport;

#[cfg(test)]
pub mod test {
    pub fn source_id() -> String {
        String::from("test source id")
    }
}
