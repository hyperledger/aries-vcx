#![deny(clippy::unwrap_used)]

#[macro_use]
extern crate lazy_static;

pub mod httpclient;
pub mod errors;
pub mod testing;
pub mod validation;
