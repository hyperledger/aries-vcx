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
pub mod delayed_serde;
mod error;
pub mod message_type;
pub mod mime_type;
pub mod protocols;
mod utils;
