#![cfg_attr(feature = "fatal_warnings", deny(warnings))]
#![crate_name = "aries_vcx"]
//this is needed for some large json macro invocations
#![recursion_limit = "128"]
pub extern crate agency_client;

pub extern crate vdrtools_sys;
pub extern crate vdrtoolsrs as vdrtools;

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

pub extern crate messages;

#[macro_use]
pub mod utils;

#[macro_use]
pub mod handlers;

pub mod error;
pub mod global;
pub mod indy;
pub mod protocols;

pub mod core;
pub mod plugins;
pub mod xyz;

#[cfg(test)]
pub mod test {
    pub fn source_id() -> String {
        String::from("test source id")
    }
}
