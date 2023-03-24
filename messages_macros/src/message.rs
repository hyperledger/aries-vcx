#![allow(clippy::expect_fun_call)]

use std::str::FromStr;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, Error, Lit, Meta, Result as SynResult};

const MESSAGE: &str = "message";
const KIND: &str = "kind";

pub fn message_impl(input: DeriveInput) -> SynResult<TokenStream> {
    let name = input.ident;

    let mut attr_iter = input.attrs.into_iter().filter(|a| a.path.is_ident(MESSAGE));
    // Look for our attribute
    let attr = attr_iter.next().expect(&format!("must use \"{MESSAGE}\" attribute"));

    // Should be the only occurrence, otherwise panic
    if let Some(attr) = attr_iter.next() {
        return Err(Error::new(attr.span(), format!("duplicate \"{MESSAGE}\" attribute")));
    }

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
    let kind = value
        .split(':')
        .next()
        .ok_or_else(|| Error::new(s.span(), "could not parse message type"))?;

    // Make the string a Rust comprehensible type
    let kind = Ident::new(kind, s.span());

    let expanded = quote! {
        impl ConcreteMessage for #name {
            type Kind = #kind;

            fn kind() -> Self::Kind {
                #expr
            }
        }
    };

    Ok(expanded)
}
