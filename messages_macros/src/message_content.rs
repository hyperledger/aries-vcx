use darling::FromDeriveInput;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{DeriveInput, ExprPath, Generics, Result as SynResult};

#[derive(FromDeriveInput)]
#[darling(attributes(message))]
struct MessageContent {
    ident: Ident,
    generics: Generics,
    #[darling(rename = "kind")]
    type_expr: ExprPath,
}

pub fn message_content_impl(input: DeriveInput) -> SynResult<TokenStream> {
    let MessageContent {
        ident,
        generics,
        type_expr,
    } = MessageContent::from_derive_input(&input)?;

    let Generics {
        params, where_clause, ..
    } = generics;

    // Get the type of the message kind
    // We don't expect paths here, but rather
    // an instance of an enum with unit variants.
    //
    // E.g: A::B
    let ty = type_expr
        .path
        .segments
        .iter()
        .next()
        .expect("at least one iteration")
        .into_token_stream();

    let expanded = quote! {
        impl<#params> crate::protocols::traits::ConcreteMessage for #ident<#params>
        #where_clause
        {
            type Kind = #ty;

            fn kind() -> Self::Kind {
                #type_expr
            }
        }
    };

    Ok(expanded)
}
