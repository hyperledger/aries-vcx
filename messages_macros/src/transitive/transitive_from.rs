#![allow(clippy::expect_fun_call)]

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Path, Result as SynResult};

use super::{validate_attr_args, MinimalAttrArgs, TRANSITIVE, TRANSITIVE_ALL};

/// Processes an attribute based on its kind
pub fn transitive_from_process_attr(name: &Ident, attr: Attribute) -> Option<SynResult<TokenStream>> {
    if attr.path.is_ident(TRANSITIVE) {
        Some(process_transitive_attr(name, attr))
    } else if attr.path.is_ident(TRANSITIVE_ALL) {
        Some(process_transitive_all_attr(name, attr))
    } else {
        None
    }
}

/// Parses attribute's parameters and returns a [`TokenStream`]
/// containing a single [`From`] impl, from `name` to the last argument of the attribute.
fn process_transitive_attr(name: &Ident, attr: Attribute) -> SynResult<TokenStream> {
    let MinimalAttrArgs { first, mut last, iter } = validate_attr_args(attr)?;

    // Create the buffer and store the minimum amount of statements.
    let mut stmts = TokenStream::new();
    stmts.extend(quote! {let interm = #first::from(val);});
    stmts.extend(quote! {let interm = #last::from(interm);});

    // Store other statements, if any
    for param in iter {
        last = param?;
        stmts.extend(quote! {let interm = #last::from(interm);});
    }

    // Generate code
    let expanded = quote! {
        impl From<#name> for #last {
            fn from(val: #name) -> #last {
                #stmts
                interm
            }
        }
    };

    Ok(expanded)
}

/// Parses the attribute's arguments and returns a [`TokenStream`]
/// containing [`From`] impls between the derived type and each two successive given arguments.
fn process_transitive_all_attr(name: &Ident, attr: Attribute) -> SynResult<TokenStream> {
    let MinimalAttrArgs {
        mut first,
        mut last,
        iter,
    } = validate_attr_args(attr)?;

    // Create the buffer and store the first impl.
    let mut impls = TokenStream::new();
    impls.extend(create_from_impl(name, &first, &last));

    // Create and store other impls, if any
    for param in iter {
        first = last;
        last = param?;
        impls.extend(create_from_impl(name, &first, &last));
    }

    Ok(impls)
}

fn create_from_impl(name: &Ident, interm: &Path, target: &Path) -> TokenStream {
    quote! {
        impl From<#name> for #target {
            fn from(val: #name) -> #target {
                let interm = #interm::from(val);
                #target::from(interm)
            }
        }
    }
}
