#![allow(clippy::expect_fun_call)]

mod message;
mod transient_from;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Error};
use transient_from::transient_from_impl;

#[proc_macro_derive(Message)]
pub fn message(_input: TokenStream) -> TokenStream {
    let expanded = quote! {};

    TokenStream::from(expanded)
}

#[proc_macro_derive(TransientFrom, attributes(transient_from))]
pub fn transient_from(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    transient_from_impl(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
