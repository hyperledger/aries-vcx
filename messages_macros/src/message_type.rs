#![allow(clippy::expect_fun_call)]

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated, spanned::Spanned, Data, DeriveInput, Error, Field, Fields, Lit, LitStr, Meta, MetaList,
    MetaNameValue, NestedMeta, Path, Result as SynResult, Token, Variant,
};

use crate::common::{end_or_err, next_or_err, next_or_panic};

const SEMVER: &str = "semver";
const MINOR: &str = "minor";
const MAJOR: &str = "major";
const FAMILY: &str = "family";
const PARENT: &str = "parent";
const ACTORS: &str = "actors";

pub fn message_type_impl(input: DeriveInput) -> SynResult<TokenStream> {
    let name = input.ident;

    let mut attr_iter = input.attrs.into_iter().filter(|a| a.path.is_ident(SEMVER));

    // Look for our attribute
    let attr = next_or_panic(&mut attr_iter, &format!("must use \"{SEMVER}\" attribute"));

    // Should be the only occurrence
    end_or_err(&mut attr_iter, format!("duplicate \"{SEMVER}\" attribute"))?;

    // Should be a list of name value pairs
    let list: Punctuated<_, Token![,]> = attr.parse_args_with(Punctuated::parse_terminated)?;

    let mut iter = list.into_iter();
    let nv = try_get_nv_pair(&mut iter, attr.span())?;

    // Process arguments
    let ts = if nv.path.is_ident(MINOR) {
        let parent = try_get_parent(&mut iter, attr.span())?;
        process_minor(&name, parent, nv)
    } else if nv.path.is_ident(MAJOR) {
        let parent = try_get_parent(&mut iter, attr.span())?;
        let actors = try_get_actors(&mut iter, attr.span())?;
        process_major(&name, parent, actors, nv, input.data)
    } else if nv.path.is_ident(FAMILY) {
        process_family(&name, nv, input.data)
    } else {
        Err(Error::new(
            nv.path.span(),
            format!("expecting (\"{FAMILY}\"), (\"{MAJOR}\", \"{PARENT}\") or (\"{MINOR}\", \"{PARENT}\")"),
        ))
    }?;

    // Error on other arguments provided
    end_or_err(&mut iter, "too many arguments")?;

    Ok(ts)
}

/// Matches the next value from the iter to get a name value pair.
fn try_get_nv_pair<I>(iter: &mut I, span: Span) -> SynResult<MetaNameValue>
where
    I: Iterator<Item = Meta>,
{
    match next_or_err(iter, span, "expecting arguments")? {
        Meta::NameValue(nv) => Ok(nv),
        v => Err(Error::new(v.span(), "expecting name value pair")),
    }
}

/// Matches the next value from the iter to get a list
fn try_get_list<I>(iter: &mut I, span: Span) -> SynResult<MetaList>
where
    I: Iterator<Item = Meta>,
{
    match next_or_err(iter, span, "expecting arguments")? {
        Meta::List(list) => Ok(list),
        v => Err(Error::new(v.span(), "expecting a list")),
    }
}

fn try_get_lit_str(nested: NestedMeta) -> SynResult<LitStr> {
    match nested {
        NestedMeta::Lit(Lit::Str(l)) => Ok(l),
        v => Err(Error::new(v.span(), "values must be literal strings")),
    }
}

/// Matches the next name value pair from the iter to get a path to a parent type.
fn try_get_parent<I>(iter: &mut I, span: Span) -> SynResult<Path>
where
    I: Iterator<Item = Meta>,
{
    let parent = try_get_nv_pair(iter, span)?;

    if parent.path.is_ident(PARENT) {
        match parent.lit {
            Lit::Str(s) => Ok(s.parse()?),
            l => Err(Error::new(l.span(), "expecting literal string")),
        }
    } else {
        Err(Error::new(parent.span(), format!("missing \"{PARENT}\" argument")))
    }
}

/// Matches the next name value pair from the iter to get a list of actors.
fn try_get_actors<I>(iter: &mut I, span: Span) -> SynResult<Punctuated<NestedMeta, Token![,]>>
where
    I: Iterator<Item = Meta>,
{
    let actors = try_get_list(iter, span)?;

    if actors.path.is_ident(ACTORS) {
        Ok(actors.nested)
    } else {
        Err(Error::new(actors.span(), format!("missing \"{ACTORS}\" argument")))
    }
}

/// Iterates the variant and returns it's name and field if it only has one.
fn try_get_var_parts(var: Variant) -> SynResult<(Ident, Field)> {
    let var_name = var.ident;

    let var_content = match var.fields {
        Fields::Unnamed(v) => v,
        e => return Err(Error::new(e.span(), "only tuple variants allowed")),
    };

    let mut iter = var_content.unnamed.into_iter();
    let field = next_or_err(&mut iter, var_name.span(), "variants must have exactly one field")?;
    end_or_err(&mut iter, "variants must have exactly one field")?;

    Ok((var_name, field))
}

fn process_minor(name: &Ident, parent: Path, minor: MetaNameValue) -> SynResult<TokenStream> {
    // Ensure the value provided is an integer
    let Lit::Int(i) = minor.lit else {
        return Err(Error::new(minor.lit.span(), "expecting u8"));
    };

    let expanded = quote! {
        impl ResolveMsgKind for #name {
            type Parent = #parent;
            const MINOR: u8 = #i;
        }
    };

    Ok(expanded)
}

fn process_major(
    name: &Ident,
    parent: Path,
    actors: Punctuated<NestedMeta, Token![,]>,
    major: MetaNameValue,
    data: Data,
) -> SynResult<TokenStream> {
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
        let (var_name, field) = try_get_var_parts(var)?;

        resolve_fn_match.extend(quote! {#field::MINOR => Ok(Self::#var_name(#field::resolve_kind(kind)?)),});
        as_parts_fn_match.extend(quote! {Self::#var_name(v) => v.as_minor_ver_parts(),});
    }

    let num_actors = actors.len();
    let actors: Vec<_> = actors.into_iter().map(try_get_lit_str).collect::<SynResult<_>>()?;

    let expanded = quote! {
        impl ResolveMinorVersion for #name {
            type Actors = [&'static str; #num_actors];
            type Parent = #parent;
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

            fn actors() -> Self::Actors {
                [#(#actors),*]
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
        let (var_name, field) = try_get_var_parts(var)?;

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
