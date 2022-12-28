#![allow(clippy::or_fun_call)]
#![allow(clippy::module_inception)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::new_without_default)]
#![allow(clippy::inherent_to_string)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::type_complexity)]
#![allow(clippy::await_holding_lock)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
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

#[cfg(feature = "libvcx_c")]
#[macro_use]
extern crate cfg_if;

#[macro_use]
pub mod api_vcx;

#[cfg(feature = "libvcx_c")]
pub mod api_c;
pub mod errors;
