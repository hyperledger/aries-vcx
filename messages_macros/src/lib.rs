#![allow(clippy::expect_fun_call)]

mod message;
mod message_type;
mod common;

use message::message_impl;
use message_type::message_type_impl;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Error};

/// Derive macro to be used on actual message that can be received.
/// The macro simplifies mapping the message to its respective message type.
///
/// It accepts a single instance of the `message` attribute, followed by
/// a string literal that maps to a *value* of the message type.
///
/// ``` ignore
/// use messages_macros::Message;
///
/// enum A {
///   Variant1,
///   Variant2,
///   Variant3
/// }
///
/// #[derive(Message)]
/// #[message("A::Variant2")]
/// struct B;
/// ```
#[proc_macro_derive(MessageContent, attributes(message))]
pub fn message_content(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    message_impl(input).unwrap_or_else(Error::into_compile_error).into()
}

/// Derive macro to be used for easier implementation of message type components.
/// The macro serves as implementation for semver reasoning and parsing of the `@type` field
/// components (major, minor versions, message kind, etc.) in a message.
///
/// The macro supports the a single instance of the `semver` attribute,
/// with a single name-value pair.
///
/// The attribute's argument can be *ONLY ONE* of the following are:
/// * family -> literal string representing the message family
/// * major -> u8 representing the protocol's major version
/// * minor -> u8 representing the protocol's minor version
///
/// ``` ignore
/// use messages_macros::MessageType;
///
/// #[derive(MessageType)]
/// #[semver(family = "some_protocol")]
/// enum A {
///    B(B)
/// };
///
/// #[derive(MessageType)]
/// #[semver(major = 1)]
/// enum B {
///    C(C)
/// };
///
/// #[derive(MessageType)]
/// #[semver(minor = 1)]
/// struct C;
/// ```
#[proc_macro_derive(MessageType, attributes(semver))]
pub fn message_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    message_type_impl(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
