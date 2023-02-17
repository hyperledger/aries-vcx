#![allow(clippy::expect_fun_call)]

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput, Error, Fields, Lit, Meta, MetaNameValue, Result as SynResult};

const SEMVER: &str = "semver";
const MINOR: &str = "minor";
const MAJOR: &str = "major";
const FAMILY: &str = "family";

pub fn message_type_impl(input: DeriveInput) -> SynResult<TokenStream> {
    let name = input.ident;

    let mut attr_iter = input.attrs.into_iter().filter(|a| a.path.is_ident(SEMVER));
    // Look for our attribute
    let attr = attr_iter.next().expect(&format!("must use \"{SEMVER}\" attribute"));

    // Should be the only occurrence, otherwise panic
    if let Some(attr) = attr_iter.next() {
        return Err(Error::new(attr.span(), format!("duplicate \"{SEMVER}\" attribute")));
    }

    // Should be a name value pair
    let Meta::NameValue(nv) = attr.parse_args()? else {
            return Err(Error::new(attr.span(), format!("expecting a single \"{FAMILY}\", \"{MAJOR}\" or \"{MINOR}\" arguments")));
        };

    if nv.path.is_ident(MINOR) {
        process_minor(&name, nv)
    } else if nv.path.is_ident(MAJOR) {
        process_major(&name, nv, input.data)
    } else if nv.path.is_ident(FAMILY) {
        process_family(&name, nv, input.data)
    } else {
        return Err(Error::new(
            nv.path.span(),
            format!("expecting a single \"{FAMILY}\", \"{MAJOR}\" or \"{MINOR}\" arguments"),
        ));
    }
}

fn process_minor(name: &Ident, minor: MetaNameValue) -> SynResult<TokenStream> {
    // Ensure the value provided is an integer
    let Lit::Int(i) = minor.lit else {
        return Err(Error::new(minor.lit.span(), "expecting u8"));
    };

    let expanded = quote! {
        impl ResolveMsgKind for #name {
            const MINOR: u8 = #i;
        }
    };

    Ok(expanded)
}

fn process_major(name: &Ident, major: MetaNameValue, data: Data) -> SynResult<TokenStream> {
    // Ensure the value provided is an integer
    let Lit::Int(i) = major.lit else {
        return Err(Error::new(major.lit.span(), "expecting u8"));
    };

    let Data::Enum(d) = data else {
        return Err(Error::new(name.span(), "can only derive on enums"));
    };

    let mut resolve_fn_match = TokenStream::new();
    let mut as_parts_fn_match = TokenStream::new();

    for var in d.variants {
        let var_name = var.ident;
        let Fields::Unnamed(var_content) = var.fields else {
            return Err(Error::new(var_name.span(), "only tuple variants allowed"));
        };

        let mut fields_iter = var_content.unnamed.into_iter();

        let Some(field) = fields_iter.next() else {
            return Err(Error::new(var_name.span(), "variants must have exactly one field"));
        };

        if let Some(f) = fields_iter.next() {
            return Err(Error::new(f.span(), "variants must have exactly one field"));
        }

        resolve_fn_match.extend(quote! {#field::MINOR => Ok(Self::#var_name(#field::resolve_kind(kind)?)),});
        as_parts_fn_match.extend(quote! {Self::#var_name(v) => v.as_minor_ver_parts(),});
    }

    let expanded = quote! {
        impl ResolveMinorVersion for #name {
            const MAJOR: u8 = #i;

            fn resolve_minor_ver(minor: u8, kind: &str) -> MsgTypeResult<Self> {
                match minor {
                    #resolve_fn_match
                    _ => Err(MsgTypeError::minor_ver_err(minor)),
                }
            }

            fn as_full_ver_parts(&self) -> (u8, u8, &str) {
                let (minor, kind) = match self {
                    #as_parts_fn_match
                };

                (Self::MAJOR, minor, kind)
            }
        }
    };

    Ok(expanded)
}

fn process_family(name: &Ident, family: MetaNameValue, data: Data) -> SynResult<TokenStream> {
    // Ensure the value provided is an integer
    let Lit::Str(s) = family.lit else {
        return Err(Error::new(family.lit.span(), "expecting literal"));
    };

    let Data::Enum(d) = data else {
        return Err(Error::new(name.span(), "can only derive on enums"));
    };

    let mut resolve_fn_match = TokenStream::new();
    let mut as_parts_fn_match = TokenStream::new();

    for var in d.variants {
        let var_name = var.ident;
        let Fields::Unnamed(var_content) = var.fields else {
            return Err(Error::new(var_name.span(), "only tuple variants allowed"));
        };

        let mut fields_iter = var_content.unnamed.into_iter();

        let Some(field) = fields_iter.next() else {
            return Err(Error::new(var_name.span(), "variants must have exactly one field"));
        };

        if let Some(f) = fields_iter.next() {
            return Err(Error::new(f.span(), "variants must have exactly one field"));
        }

        resolve_fn_match
            .extend(quote! {#field::MAJOR => Ok(Self::#var_name(#field::resolve_minor_ver(minor, kind)?)),});
        as_parts_fn_match.extend(quote! {Self::#var_name(v) => v.as_full_ver_parts(),});
    }

    let expanded = quote! {
        impl ResolveMajorVersion for #name {
            const FAMILY: &'static str = #s;

            fn resolve_major_ver(major: u8, minor: u8, kind: &str) -> MsgTypeResult<Self> {
                match major {
                    #resolve_fn_match
                    _ => Err(MsgTypeError::major_ver_err(major)),
                }
            }

            fn as_msg_type_parts(&self) -> (&str, u8, u8, &str) {
                let (major, minor, kind) = match self {
                    #as_parts_fn_match
                };

                (Self::FAMILY, major, minor, kind)
            }
        }
    };

    Ok(expanded)
}
