#![allow(clippy::expect_fun_call)]

mod message;
mod transitive_from;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Error};
use transitive_from::transitive_from_impl;

#[proc_macro_derive(Message)]
pub fn message(_input: TokenStream) -> TokenStream {
    let expanded = quote! {};

    TokenStream::from(expanded)
}

/// Derive macro that implements [From] for A -> C by converting A -> B -> C.
/// For this to work, [`From`] A to B and [`From`] B to C impls must exist.
/// The attribute is where the list of types to transit through is provided, in order.
///
/// Multiple attributes can be used on a single type for multiple transitive [`From`] impls
/// and the trasitions chain is virtually unlimited.
/// ``` ignore
/// use crate::TransitiveFrom;
///
/// #[derive(TransitiveFrom)]
/// #[transitive_from(B, C, D, E, F)] // impl From<A> for F
/// struct A;
/// #[derive(TransitiveFrom)]
/// #[transitive_from(C, D, E)] // impl From<B> for E
/// #[transitive_from(E, F)] // impl From<B> for F => Since we already implement B -> E above, this works!
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
#[proc_macro_derive(TransitiveFrom, attributes(transitive_from))]
pub fn transitive_from(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    transitive_from_impl(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
