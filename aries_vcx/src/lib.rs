#![cfg_attr(feature = "fatal_warnings", deny(warnings))]
#![crate_name = "aries_vcx"]
//this is needed for some large json macro invocations
#![recursion_limit = "128"]
pub extern crate agency_client;
extern crate base64;
extern crate chrono;
extern crate failure;
extern crate futures;
pub extern crate vdrtools_sys;
pub extern crate vdrtoolsrs as vdrtools;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
extern crate openssl;
extern crate rand;
extern crate regex;
extern crate rmp_serde;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate strum;
extern crate strum_macros;
extern crate time;
extern crate url;
extern crate uuid;
#[macro_use]
extern crate derive_builder;
extern crate messages;

#[macro_use]
pub mod utils;
#[macro_use]
pub mod handlers;
pub mod error;
pub mod global;
pub mod libindy;
pub mod protocols;

#[cfg(test)]
pub mod test {
    pub fn source_id() -> String {
        String::from("test source id")
    }
}
