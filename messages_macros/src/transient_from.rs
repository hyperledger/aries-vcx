#![allow(clippy::expect_fun_call)]

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    punctuated::Punctuated, spanned::Spanned, Attribute, DeriveInput, Error, Lit, MetaNameValue, Path,
    Result as SynResult, Token,
};

const TARGET: &str = "target";
const PARENT: &str = "parent";
const GRANDPARENT: &str = "grandparent";
const TRANSIENT_FROM: &str = "transient_from";

pub fn transient_from_impl(input: DeriveInput) -> SynResult<TokenStream2> {
    // Name of the type this gets derived on
    let name = input.ident;

    // Storage for the parameters we accept:
    let mut target: Option<Path> = None;
    let mut parent: Option<Path> = None;
    let mut grandparent: Option<Path> = None;

    // The attribute paired with the derive
    let attr = try_find_attr(input.attrs)?;
    let parameters: Punctuated<MetaNameValue, Token![,]> = attr.parse_args_with(Punctuated::parse_terminated)?;

    // Parse parameters and store them
    // We only accept one of each
    for MetaNameValue { path, lit, .. } in parameters {
        if path.is_ident(TARGET) {
            process_parameter(path, lit, &mut target, TARGET)?;
        } else if path.is_ident(PARENT) {
            process_parameter(path, lit, &mut parent, PARENT)?;
        } else if path.is_ident(GRANDPARENT) {
            process_parameter(path, lit, &mut grandparent, GRANDPARENT)?;
        } else { // We don't accept unknown parameters
            return Err(Error::new(
                path.span(),
                format!("unknown parameter \"{}\"", path.to_token_stream()),
            ));
        }
    }

    // Ensure these are populated
    let target = check_parameter(target, attr.path.span(), TARGET)?;
    let parent = check_parameter(parent, attr.path.span(), PARENT)?;
    
    // Ignore if None
    let grandparent_stmt = match grandparent {
        Some(gp) => quote! {let interm = #gp::from(interm);},
        None => quote! {},
    };

    // Generate the output TokenStream
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

fn try_find_attr(attrs: Vec<Attribute>) -> SynResult<Attribute> {
    let mut storage = None;

    for attr in attrs {
        // If this is our attr:
        if attr.path.is_ident(TRANSIENT_FROM) {
            if storage.is_none() { // This is the first attribute found
                storage = Some(attr);
            } else { // Duplicate attribute encountered
                return Err(Error::new(
                    attr.span(),
                    format!("only one\"{TRANSIENT_FROM}\" attribute allowed"),
                ));
            }
        }
    }

    // Panicking here results in a compile time error on the derived attribute, which is what we want.
    // There's no better span since there's no attribute to use as reference.
    let attr = storage.expect(&format!("must use the \"{TRANSIENT_FROM}\" attribute"));
    Ok(attr)
}

fn process_parameter(path: Path, value: Lit, storage: &mut Option<Path>, name: &str) -> SynResult<()> {
    // Error out if we already got this parameter
    if storage.is_some() {
        return Err(Error::new(path.span(), format!("duplicate \"{name}\" parameter")));
    }

    // We only want literals that represent a type's path
    let Lit::Str(ref s) = value else {return Err(Error::new(value.span(), "expecting literal"))};
    *storage = Some(s.parse()?);

    Ok(())
}

fn check_parameter(opt: Option<Path>, span: Span, name: &str) -> SynResult<Path> {
    opt.ok_or_else(|| Error::new(span, format!("missing \"{name}\" parameter")))
}
