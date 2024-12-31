#![allow(clippy::or_fun_call)]
#![allow(clippy::module_inception)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::new_without_default)]
#![allow(clippy::inherent_to_string)]
#![allow(clippy::large_enum_variant)]

#[macro_use]
extern crate serde;
extern crate shared;
pub mod aries;
pub mod errors;
pub mod w3c;
