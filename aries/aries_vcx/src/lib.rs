#![allow(clippy::result_large_err)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::diverging_sub_expression)]
//this is needed for some large json macro invocations
#![recursion_limit = "128"]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate derive_builder;

pub extern crate did_doc;
pub extern crate did_parser_nom;
pub extern crate did_peer;
pub extern crate messages;

pub use aries_vcx_anoncreds;
pub use aries_vcx_wallet;

#[macro_use]
pub mod utils;

#[macro_use]
pub mod handlers;

pub mod global;
pub mod protocols;

pub mod common;
pub mod errors;
pub mod transport;
