extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(Display)]
pub fn display_as_json_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_display(&ast)
}

fn impl_display(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl std::fmt::Display for #name {
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
