#![allow(clippy::expect_fun_call)]

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    punctuated::Punctuated, spanned::Spanned, Attribute, DeriveInput, Error, Meta, NestedMeta, Path,
    Result as SynResult, Token,
};

const TRANSITIVE_FROM: &str = "transitive_from";

pub fn transitive_from_impl(input: DeriveInput) -> SynResult<TokenStream> {
    let name = input.ident;

    // An iterator of [`TokenStream`].
    // Each of them is a set of statements for one [`From`] impl.
    let params_iter = input
        .attrs
        .into_iter()
        .filter(|a| a.path.is_ident(TRANSITIVE_FROM))
        .map(parse_params);

    let mut expanded = TokenStream::new();

    // For each set of statements, create the [`From`] impl and store it in the buffer.
    for params in params_iter {
        let (params, target) = params?;
        let from_impl = quote! {
            impl From<#name> for #target {
                fn from(val: #name) -> #target {
                    #params
                    interm
                }
            }
        };
        expanded.extend(from_impl);
    }

    Ok(expanded)
}

/// Parses attribute's parameters and returns a [`TokenStream`] along
/// with the last parameter found (last type provided).
///
/// The last parameter is needed for defining the [`From`] impl.
fn parse_params(attr: Attribute) -> SynResult<(TokenStream, Path)> {
    // Save the span in case we issue errors.
    // Consuming the attribute arguments prevents us from doing that later.
    let attr_span = attr.span();

    // Parse arguments and create an iterator of [`Path`] (types) items.
    let mut iter = attr
        .parse_args_with(Punctuated::<NestedMeta, Token![,]>::parse_terminated)?
        .into_iter()
        .map(validate_param);

    // Ensure we were provided with at least two elements.
    let (first, mut last) = match (iter.next(), iter.next()) {
        (Some(first), Some(last)) => Ok((first?, last?)),
        _ => Err(Error::new(attr_span, "at least two parameters needed")),
    }?;

    // Create the buffer and store the minimum amount of statements.
    let mut stmts = TokenStream::new();
    stmts.extend(quote! {let interm = #first::from(val);});
    stmts.extend(quote! {let interm = #last::from(interm);});

    // Store other statements
    for param in iter {
        last = param?;
        stmts.extend(quote! {let interm = #last::from(interm);});
    }

    Ok((stmts, last))
}

/// Ensures we only accept types, not literals, integers or anything like that.
fn validate_param(param: NestedMeta) -> SynResult<Path> {
    match param {
        NestedMeta::Meta(Meta::Path(p)) => Ok(p),
        _ => Err(Error::new(param.span(), "only type paths accepted")),
    }
}
