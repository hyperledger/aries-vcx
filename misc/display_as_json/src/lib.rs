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

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // For each of the generics parameter G, we prepare addition trait bound "G: serde::Serialize"
    let serialize_bounds = generics.params.iter().filter_map(|param| {
        match param {
            syn::GenericParam::Type(type_param) => {
                let ident = &type_param.ident;
                Some(quote! { #ident: serde::Serialize })
            }
            _ => None, // Skip non-type parameters, eg lifetimes
        }
    });

    // Combine the original where clause with additional bounds we prepared above
    // See quote! macro docs for more info on the #() syntax https://docs.rs/quote/1.0.33/quote/macro.quote.html
    let combined_where_clause = if where_clause.is_none() {
        quote! { where #(#serialize_bounds),* }
    } else {
        quote! { #where_clause, #(#serialize_bounds),* }
    };

    // Generate the actual impl
    let gen = quote! {
        impl #impl_generics std::fmt::Display for #name #ty_generics #combined_where_clause {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match serde_json::to_string(self) {
                    Ok(json) => write!(f, "{}", json),
                    Err(e) => write!(f, "Error serializing {}: {}", stringify!(#name), e),
                }
            }
        }
    };
    gen.into()
}
