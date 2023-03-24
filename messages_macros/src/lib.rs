#![allow(clippy::expect_fun_call)]

mod message_type;

use message_type::message_type_impl;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Error};

/// Derive macro to be used for easier implementation of message type components.
/// The macro serves as implementation for semver reasoning and parsing of the `@type` field
/// components (major, minor versions, message kind, etc.) in a message.
///
/// The macro can only be derived on *newtype enums*, expecting either
/// a protocol or a major version encapsulating enum.
///
/// The minor versions are represented by the major version enum's variants
/// and the field encapsulated in the variants are expected to be [`std::marker::PhantomData<fn() ->
/// T>`]. The `T` binds the message kinds of the protocol to the minor version variant of the enum.
///
/// As a summary, this macro will generate the following:
/// - on protocol representing enums:
///     - [`ProtocolName`] impl on the enum.
///     - regular impl on the enum containing `const PROTOCOL: &str`.
///
/// - on major version representing enums:
///     - [`ProtocolVersion`] impl on the enum.
///     - [`MessageKind`] impls on each type bound in the variants.
///     - `new_vX_Y()` shorthand methods on the enum, for easier creation of instances of a certain variant (version).
///
/// As per why the generic type is `fn() -> T` and not just `T`, the short story is *ownership*.
///
/// The long story is that `PhantomData<T>` tells the drop checker that we *own* `T`, which we
/// don't. While still a covariant, `fn() -> T` does not mean we own the `T`, so that let's the drop
/// checker be more permissive. Not really important for our current use case, but it is
/// *idiomatic*.
///
/// Good reads and references:
/// - https://doc.rust-lang.org/std/marker/struct.PhantomData.html
/// - https://doc.rust-lang.org/nomicon/phantom-data.html
/// - https://doc.rust-lang.org/nomicon/phantom-data.html
/// - https://doc.rust-lang.org/nomicon/dropck.html
///
/// ``` ignore
/// use messages_macros::MessageType;
/// use std::marker::PhantomData;
///
/// // as if used from within the `messages` crate
/// use crate::msg_types::role::Role
///
/// #[derive(MessageType)]
/// #[msg_type(protocol = "some_protocol")]
/// enum SomeProtocol {
///    V1(SomeProtocolV1)
/// };
///
/// #[derive(MessageType)]
/// #[msg_type(major = 1)]
/// enum SomeProtocolV1 {
///    #[msg_type(minor = 0, roles = "Role::Receiver, Role::Sender")]
///    V1_0(PhantomData<fn() -> SomeProtocolV1_0>)
/// };
///
/// /// The message kinds the protocol handles.
/// #[semver(minor = 1)]
/// enum SomeProtocolV1_0 {
///     Message,
///     Request,
///     Response
/// }
/// ```
#[proc_macro_derive(MessageType, attributes(msg_type))]
pub fn message_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    message_type_impl(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
