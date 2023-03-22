use darling::FromDeriveInput;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, Error, ExprPath, Generics, PathSegment, Result as SynResult};

/// Matches the input from deriving the macro on a type.
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

    let kind_type = extract_msg_kind_type(&type_expr)?;

    let expanded = quote! {
        impl<#params> crate::protocols::traits::MessageContent for #ident<#params>
        #where_clause
        {
            type Kind = #kind_type;

            fn kind() -> Self::Kind {
                #type_expr
            }
        }
    };

    Ok(expanded)
}

/// Extracts the type of the message kind.
/// We expect an enum variant expression here, like `A::B` or `crate::module::A::B`.
///
/// We need the `A` part, so the second last path segment.
fn extract_msg_kind_type(type_expr: &ExprPath) -> SynResult<&PathSegment> {
    type_expr
        .path
        .segments
        .iter()
        .nth_back(1)
        .ok_or_else(|| Error::new(type_expr.span(), "could not determine message kind's type"))
}
