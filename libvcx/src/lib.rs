#![cfg_attr(feature = "fatal_warnings", deny(warnings))]
#![crate_name = "vcx"]

//this is needed for some large json macro invocations
#![recursion_limit = "128"]
#[macro_use]
extern crate aries_vcx;

extern crate num_traits;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate cfg_if;

#[macro_use]
pub mod api_lib;
