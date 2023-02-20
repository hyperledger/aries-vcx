#![allow(clippy::expect_fun_call)]

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Path, Result as SynResult};

use super::{ validate_attr_args, MinimalAttrArgs, TRANSITIVE, TRANSITIVE_ALL};

/// Processes an attribute based on its kind
pub fn transitive_try_from_process_attr(name: &Ident, attr: Attribute) -> Option<SynResult<TokenStream>> {
    if attr.path.is_ident(TRANSITIVE) {
        Some(process_transitive_attr(name, attr))
    } else if attr.path.is_ident(TRANSITIVE_ALL) {
        Some(process_transitive_all_attr(name, attr))
    } else {
        None
    }
}

/// Parses attribute's parameters and returns a [`TokenStream`]
/// containing a single [`TryFrom`] impl, from `name` to the last argument of the attribute.
fn process_transitive_attr(name: &Ident, attr: Attribute) -> SynResult<TokenStream> {
    let MinimalAttrArgs { first, mut last, iter } = validate_attr_args(attr)?;

    // Store statements in a Vec first as we'll have to reverse them.
    let mut stmts_vec = Vec::new();
    stmts_vec.push(quote! {let interm: #name = interm.try_into()?;});
    stmts_vec.push(quote! {let interm: #first = interm.try_into()?;});

    // Store other statements, if any
    for param in iter {
        stmts_vec.push(quote! {let interm: #last = interm.try_into()?;});
        last = param?;
    }

    stmts_vec.push(quote! {let interm = val;});

    let mut stmts = TokenStream::new();
    for stmt in stmts_vec.into_iter().rev() {
        stmts.extend(stmt);
    }

    // Generate code
    let expanded = quote! {
        impl TryFrom<#last> for #name {
            type Error = <#name as TryFrom<#first>>::Error;

            fn try_from(val: #last) -> Result<Self, Self::Error> {
                #stmts
                Ok(interm)
            }
        }
    };

    Ok(expanded)
}

/// Parses the attribute's arguments and returns a [`TokenStream`]
/// containing [`TryFrom`] impls between the derived type and each two successive given arguments.
fn process_transitive_all_attr(name: &Ident, attr: Attribute) -> SynResult<TokenStream> {
    let MinimalAttrArgs {
        mut first,
        mut last,
        iter,
    } = validate_attr_args(attr)?;

    // Create the buffer and store the first impl.
    let mut impls = TokenStream::new();
    impls.extend(create_try_from_impl(name, &first, &last));

    // Create and store other impls, if any
    for param in iter {
        first = last;
        last = param?;
        impls.extend(create_try_from_impl(name, &first, &last));
    }

    Ok(impls)
}

fn create_try_from_impl(name: &Ident, interm: &Path, target: &Path) -> TokenStream {
    quote! {
        impl TryFrom<#target> for #name {
            type Error = <#name as TryFrom<#interm>>::Error;

            fn try_from(val: #target) -> Result<Self, Self::Error> {
                let interm = #interm::try_from(val)?;
                #name::try_from(interm)
            }
        }
    }
}
