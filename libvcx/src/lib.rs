#![cfg_attr(feature = "fatal_warnings", deny(warnings))]
#![crate_name = "vcx"]
//this is needed for some large json macro invocations
#![recursion_limit = "128"]
#[macro_use]
extern crate aries_vcx;
extern crate base64;
extern crate chrono;
extern crate failure;
extern crate futures;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
extern crate openssl;
extern crate rand;
extern crate rmp_serde;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate async_std;
extern crate time;
extern crate tokio;
extern crate uuid;

#[macro_use]
pub mod api_lib;
