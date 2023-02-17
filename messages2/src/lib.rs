#![allow(clippy::or_fun_call)]
#![allow(clippy::module_inception)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::new_without_default)]
#![allow(clippy::inherent_to_string)]
#![allow(clippy::large_enum_variant)]
#![deny(clippy::unwrap_used)]

mod aries_message;
mod decorators;
mod delayed_serde;
mod error;
mod message_type;
mod protocols;
