#![allow(clippy::expect_fun_call)]

mod message;
mod message_type;
mod transitive;

use message::message_impl;
use message_type::message_type_impl;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Error};
use transitive::{transitive_from_process_attr, transitive_impl, transitive_try_from_process_attr};

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
/// 
/// The macro supports two attributes, with slightly different behavior:
/// * `transitive` -> For A, B, C, D it derives `impl From<A> for D`, skipping `impl From<A> for C`
/// * `transitive_all` -> For A, B, C, D it derives `impl From<A> for D` *AND* `impl From<A> for C`
/// ``` ignore
/// use messages_macros::TransitiveFrom;
///
/// #[derive(TransitiveFrom)]
/// #[transitive(B, C, D, E, F, G)] // impl From<A> for G
/// struct A;
/// #[derive(TransitiveFrom)]
/// #[transitive_all(C, D, E, F, G)] // impl From<B> for D, E, F and G
/// struct B;
/// #[derive(TransitiveFrom)]
/// #[transitive(C, D, E, F)] // impl From<C> for F
/// #[transitive(F, G)] // impl From<C> for G => Since we already implement C -> F above, this works!
/// struct C;
/// struct D;
/// struct E;
/// struct F;
/// struct G;

/// impl From<A> for B {
///     fn from(val: A) -> B {
///         B
///     }
/// }

/// impl From<B> for C {
///     fn from(val: B) -> C {
///         C
///     }
/// }

/// impl From<C> for D {
///     fn from(val: C) -> D {
///         D
///     }
/// }

/// impl From<D> for E {
///     fn from(val: D) -> E {
///         E
///     }
/// }

/// impl From<E> for F {
///     fn from(val: E) -> F {
///         F
///     }
/// }

/// impl From<F> for G {
///     fn from(val: F) -> G {
///         G
///     }
/// }

/// G::from(A);

/// D::from(B);
/// E::from(B);
/// F::from(B);
/// G::from(B);

/// F::from(C);
/// G::from(C);
/// ```
#[proc_macro_derive(TransitiveFrom, attributes(transitive, transitive_all))]
pub fn transitive_from(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    transitive_impl(input, transitive_from_process_attr)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Derive macro that implements [TryFrom] for C -> A by converting C -> B -> A.
/// 
/// This works similarly with the [`TransitiveFrom`] derive macro, but for the purpose
/// of reusing the same attributes and type lists, it parses them *in reverse*.
/// 
/// Just like with [`TransitiveFrom`], [`TryFrom`] B to A and [`TryFrom`] C to B impls must exist.
/// The attribute is where the list of types to transit through is provided, in *reverse* order.
///
/// Multiple attributes can be used on a single type for multiple transitive [`TryFrom`] impls
/// and the trasitions chain is virtually unlimited.
/// 
/// The macro supports two attributes, with slightly different behavior:
/// * `transitive` -> For A, B, C, D it derives `impl TryFrom<D> for A`, skipping `impl TryFrom<C> for A`
/// * `transitive_all` -> For A, B, C, D it derives `impl TryFrom<D> for A` *AND* `impl TryFrom<C> for A`
/// ``` ignore
/// use messages_macros::TransitiveTryFrom;
/// 
/// #[derive(TransitiveTryFrom)]
/// #[transitive(B, C, D, E, F, G)] // impl TryFrom<G> for A
/// struct A;
/// #[derive(TransitiveTryFrom)]
/// #[transitive_all(C, D, E, F, G)] // impl TryFrom<G> for E, D, C and B
/// struct B;
/// #[derive(TransitiveTryFrom)]
/// #[transitive(D, E, F)] // impl TryFrom<F> for C
/// #[transitive(F, G)] // impl TryFrom<G> for C => Since we already implement C -> F above, this works!
/// struct C;
/// struct D;
/// struct E;
/// struct F;
/// struct G;

/// impl TryFrom<G> for F {
///     type Error = ();

///     fn try_from(val: G) -> Result<Self, Self::Error> {
///         Ok(F)
///     }
/// }

/// impl TryFrom<F> for E {
///     type Error = ();

///     fn try_from(val: F) -> Result<Self, Self::Error> {
///         Ok(E)
///     }
/// }

/// impl TryFrom<E> for D {
///     type Error = ();

///     fn try_from(val: E) -> Result<Self, Self::Error> {
///         Ok(D)
///     }
/// }

/// impl TryFrom<D> for C {
///     type Error = ();

///     fn try_from(val: D) -> Result<Self, Self::Error> {
///         Ok(C)
///     }
/// }

/// impl TryFrom<C> for B {
///     type Error = ();

///     fn try_from(val: C) -> Result<Self, Self::Error> {
///         Ok(B)
///     }
/// }

/// impl TryFrom<B> for A {
///     type Error = ();

///     fn try_from(val: B) -> Result<Self, Self::Error> {
///         Ok(A)
///     }
/// }

/// A::try_from(G);

/// B::try_from(D);
/// B::try_from(E);
/// B::try_from(F);
/// B::try_from(G);

/// C::try_from(F);
/// C::try_from(G);
/// ```
#[proc_macro_derive(TransitiveTryFrom, attributes(transitive, transitive_all))]
pub fn transitive_try_from(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    transitive_impl(input, transitive_try_from_process_attr)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
