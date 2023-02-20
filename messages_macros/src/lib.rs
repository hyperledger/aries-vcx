#![allow(clippy::expect_fun_call)]

mod message;
mod message_type;
mod transitive;

use message::message_impl;
use message_type::message_type_impl;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Error};
use transitive::{transitive_impl, transitive_from_process_attr, transitive_try_from_process_attr};

#[proc_macro_derive(Message, attributes(message))]
pub fn message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    message_impl(input).unwrap_or_else(Error::into_compile_error).into()
}

#[proc_macro_derive(MessageType, attributes(semver))]
pub fn message_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    message_type_impl(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Derive macro that implements [From] for A -> C by converting A -> B -> C.
/// For this to work, [`From`] A to B and [`From`] B to C impls must exist.
/// The attribute is where the list of types to transit through is provided, in order.
///
/// Multiple attributes can be used on a single type for multiple transitive [`From`] impls
/// and the trasitions chain is virtually unlimited.
/// ``` ignore
/// use messages_macros::TransitiveFrom;
///
/// #[derive(TransitiveFrom)]
/// #[transitive(B, C, D, E, F)] // impl From<A> for F
/// struct A;
/// #[derive(TransitiveFrom)]
/// #[transitive(C, D, E)] // impl From<B> for E
/// #[transitive(E, F)] // impl From<B> for F => Since we already implement B -> E above, this works!
/// struct B;
/// struct C;
/// struct D;
/// struct E;
/// struct F;
///
/// impl From<A> for B {
///     fn from(val: A) -> B {
///         B
///     }
/// }
///
/// impl From<B> for C {
///     fn from(val: B) -> C {
///         C
///     }
/// }
///
/// impl From<C> for D {
///     fn from(val: C) -> D {
///         D
///     }
/// }
///
/// impl From<D> for E {
///     fn from(val: D) -> E {
///         E
///     }
/// }
///
/// impl From<E> for F {
///     fn from(val: E) -> F {
///         F
///     }
/// }
///
///
/// let a = A;
/// let f = F::from(a);
///
/// let b = B;
/// let e = E::from(b);
/// let f = F::from(b);
///
/// ```
#[proc_macro_derive(TransitiveFrom, attributes(transitive, transitive_all))]
pub fn transitive_from(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    transitive_impl(input, transitive_from_process_attr).unwrap_or_else(Error::into_compile_error).into()
}

#[proc_macro_derive(TransitiveTryFrom, attributes(transitive, transitive_all))]
pub fn transitive_try_from(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    transitive_impl(input, transitive_try_from_process_attr)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
