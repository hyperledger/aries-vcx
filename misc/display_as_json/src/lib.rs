extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Display)]
pub fn display_as_json_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_display(&ast)
}

fn impl_display(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;

    // Create new where clause with Serialize bounds for each generic type
    let where_clause = generics.params.iter().map(|param| {
        let ident = match param {
            syn::GenericParam::Type(type_param) => &type_param.ident,
            _ => return quote!(), // Skip non-type parameters
        };
        quote! { #ident: serde::Serialize }
    });

    let gen = quote! {
        impl #generics std::fmt::Display for #name #generics
        where
            #(#where_clause),*
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let json = serde_json::to_string(self).unwrap_or_else(|e| {
                    format!("Error serializing {}: {}", stringify!(#name), e)
                });
                write!(f, "{}", json)
            }
        }
    };
    gen.into()
}
