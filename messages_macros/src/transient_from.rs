#![allow(clippy::expect_fun_call)]

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    punctuated::Punctuated, spanned::Spanned, DeriveInput, Error, Lit, MetaNameValue, Path, Result as SynResult, Token,
};

const TARGET: &str = "target";
const PARENT: &str = "parent";
const GRANDPARENT: &str = "grandparent";
const TRANSIENT_FROM: &str = "transient_from";

pub fn transient_from_impl(input: DeriveInput) -> SynResult<TokenStream2> {
    let name = input.ident;

    let mut target: Option<Path> = None;
    let mut parent: Option<Path> = None;
    let mut grandparent: Option<Path> = None;

    let attr = input
        .attrs
        .into_iter()
        .find(|a| a.path.is_ident(TRANSIENT_FROM))
        .expect(&format!("must use the \"{TRANSIENT_FROM}\" attribute"));

    let name_values: Punctuated<MetaNameValue, Token![,]> = attr.parse_args_with(Punctuated::parse_terminated)?;

    for MetaNameValue { path, lit, .. } in name_values {
        if path.is_ident("target") {
            process_parameter(path, lit, &mut target, TARGET)?;
        } else if path.is_ident("parent") {
            process_parameter(path, lit, &mut parent, PARENT)?;
        } else if path.is_ident("grandparent") {
            process_parameter(path, lit, &mut grandparent, GRANDPARENT)?;
        } else {
            return Err(Error::new(
                path.span(),
                format!("unknown parameter \"{}\"", path.to_token_stream()),
            ));
        }
    }

    let target = check_parameter(target, attr.path.span(), TARGET)?;
    let parent = check_parameter(parent, attr.path.span(), PARENT)?;

    let grandparent_stmt = match grandparent {
        Some(gp) => quote! {let interm = #gp::from(interm);},
        None => quote! {},
    };

    let expanded = quote! {
        impl From<#name> for #target {
            fn from(val: #name) -> #target {
                let interm = #parent::from(val);
                #grandparent_stmt
                #target::from(interm)
            }
        }
    };

    Ok(expanded)
}

fn process_parameter(path: Path, value: Lit, storage: &mut Option<Path>, name: &str) -> SynResult<()> {
    if storage.is_some() {
        return Err(Error::new(path.span(), format!("duplicate \"{name}\" parameter")));
    }

    let Lit::Str(ref s) = value else {return Err(Error::new(value.span(), "expecting literal"))};
    *storage = Some(s.parse()?);

    Ok(())
}

fn check_parameter(opt: Option<Path>, span: Span, name: &str) -> SynResult<Path> {
    opt.ok_or_else(|| Error::new(span, format!("missing \"{name}\" parameter")))
}
