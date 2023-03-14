#![allow(clippy::or_fun_call)]
#![allow(clippy::module_inception)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::new_without_default)]
#![allow(clippy::inherent_to_string)]
#![allow(clippy::large_enum_variant)]
#![deny(clippy::unwrap_used)]

pub mod aries_message;
pub mod composite_message;
pub mod decorators;
mod error;
pub mod msg_types;
pub mod protocols;
pub mod misc;
