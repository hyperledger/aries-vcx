#![allow(clippy::or_fun_call)]
#![allow(clippy::module_inception)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::new_without_default)]
#![allow(clippy::inherent_to_string)]
#![allow(clippy::large_enum_variant)]
#![deny(clippy::unwrap_used)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate strum_macros;

#[macro_use]
pub mod a2a;

#[macro_use]
pub mod concepts;

pub extern crate diddoc;

pub mod actors;
pub mod errors;
pub mod protocols;
pub mod status;
pub mod utils;
