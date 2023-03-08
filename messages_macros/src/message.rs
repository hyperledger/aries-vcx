#![allow(clippy::expect_fun_call)]

use std::str::FromStr;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, Error, Lit, Meta, Result as SynResult};

use crate::common::{end_or_err, next_or_panic};

const MESSAGE: &str = "message";
const KIND: &str = "kind";

pub fn message_impl(input: DeriveInput) -> SynResult<TokenStream> {
    let name = input.ident;
    let generics = input.generics;
    let params = generics.params;
    let where_clause = generics.where_clause;

    let mut attr_iter = input.attrs.into_iter().filter(|a| a.path.is_ident(MESSAGE));
    // Look for our attribute
    let attr = next_or_panic(&mut attr_iter, &format!("must use \"{MESSAGE}\" attribute"));

    // Should be the only occurrence
    end_or_err(&mut attr_iter, format!("duplicate \"{MESSAGE}\" attribute"))?;

    // Should be a name value pair
    let Meta::NameValue(nv) = attr.parse_args()? else {
        return Err(Error::new(attr.span(), format!("expecting \"{KIND}\" name value pair")));
    };

    // Ensure the argument name is right
    if !nv.path.is_ident(KIND) {
        return Err(Error::new(nv.path.span(), format!("expecting \"{KIND}\" argument")));
    }

    // Ensure the value provided is a string literal
    let Lit::Str(s) = nv.lit else {
        return Err(Error::new(nv.lit.span(), "expecting literal"));
    };

    // Parse the value to a token stream of an expression that Rust understands
    let value = s.value();
    let expr = TokenStream::from_str(&value)?;

    // Get the type of the message kind
    // We don't expect paths here, but rather
    // nested enums that end in a unit variant.
    //
    // E.g: A::B(B::C(C::D))
    let kind = value
        .split(':')
        .next()
        .ok_or_else(|| Error::new(s.span(), "could not parse message type"))?;

    // Make the string a Rust comprehensible type
    let kind = Ident::new(kind, s.span());

    let expanded = quote! {
        impl<#params> MessageKind for #name<#params>
        #where_clause
        {
            type Kind = #kind;

            fn kind() -> Self::Kind {
                #expr
            }
        }
    };

    Ok(expanded)
}
