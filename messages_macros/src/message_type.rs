#![allow(clippy::expect_fun_call)]

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated, spanned::Spanned, Data, DeriveInput, Error, Field, Fields, Lit, Meta, MetaList,
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
    let span = attr.span();

    // Should be the only occurrence
    end_or_err(&mut attr_iter, format!("duplicate \"{SEMVER}\" attribute"))?;

    // Should be a list of name value pairs
    let list: Punctuated<_, Token![,]> = attr.parse_args_with(Punctuated::parse_terminated)?;

    let mut iter = list.into_iter();
    let nv = try_get_nv_pair(&mut iter, span)?;

    // Process arguments
    let ts = if nv.path.is_ident(PARENT) {
        let parent = try_parse_parent(nv)?;
        Ok(process_kind(&name, parent))
    } else if nv.path.is_ident(MINOR) {
        let parent = try_get_nv_pair(&mut iter, span)?;
        let parent = try_parse_parent(parent)?;
        process_minor(&name, parent, nv)
    } else if nv.path.is_ident(MAJOR) {
        let parent = try_get_nv_pair(&mut iter, span)?;
        let parent = try_parse_parent(parent)?;
        let actors = try_get_actors(&mut iter, span)?;
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

/// Matches the next value to a path, to be used as enum declaration
fn try_get_path(nested: NestedMeta) -> SynResult<Path> {
    match nested {
        NestedMeta::Meta(Meta::Path(l)) => Ok(l),
        v => Err(Error::new(v.span(), "values must be enum variants")),
    }
}

/// Attempts to parse a type path from a name value pair.
fn try_parse_parent(parent: MetaNameValue) -> SynResult<Path> {
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
fn try_get_actors<I>(iter: &mut I, span: Span) -> SynResult<Vec<Path>>
where
    I: Iterator<Item = Meta>,
{
    let actors = try_get_list(iter, span)?;

    if actors.path.is_ident(ACTORS) {
        actors.nested.into_iter().map(try_get_path).collect()
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

fn process_kind(name: &Ident, parent: Path) -> TokenStream {
    quote! {
        impl MessageKind for #name {
            type Parent = #parent;

            fn parent() -> Self::Parent {
                #parent
            }
        }
    }
}

fn process_minor(name: &Ident, parent: Path, minor: MetaNameValue) -> SynResult<TokenStream> {
    // Ensure the value provided is an integer
    let Lit::Int(i) = minor.lit else {
        return Err(Error::new(minor.lit.span(), "expecting u8"));
    };

    let expanded = quote! {
        impl MinorVersion for #name {
            type Parent = #parent;

            const MINOR: u8 = #i;
        }
    };

    Ok(expanded)
}

fn process_major(
    name: &Ident,
    parent: Path,
    actors: Vec<Path>,
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

        resolve_fn_match.extend(quote! {#field::MINOR => Ok(Self::#var_name(#field)),});
        as_parts_fn_match.extend(quote! {Self::#var_name(v) => v.as_minor_version(),});
    }

    let num_actors = actors.len();

    let expanded = quote! {
        impl MajorVersion for #name {
            type Actors = [Actor; #num_actors];

            type Parent = #parent;

            const MAJOR: u8 = #i;

            fn resolve_minor_ver(minor: u8) -> MsgTypeResult<Self> {
                match minor {
                    #resolve_fn_match
                    _ => {
                        let family = Self::Parent::FAMILY;
                        let major = Self::MAJOR;
                        match get_supported_version(family, major, minor) {
                            Some(minor) => Self::resolve_minor_ver(minor),
                            None => Err(MsgTypeError::minor_ver_err(minor))
                        }
                    },
                }
            }

            fn as_version_parts(&self) -> (u8, u8) {
                let minor = match self {
                    #as_parts_fn_match
                };

                (Self::MAJOR, minor)
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

        resolve_fn_match.extend(quote! {#field::MAJOR => Ok(Self::#var_name(#field::resolve_minor_ver(minor)?)),});
        as_parts_fn_match.extend(quote! {Self::#var_name(v) => v.as_version_parts(),});
    }

    let expanded = quote! {
        impl ProtocolName for #name {
            const FAMILY: &'static str = #s;

            fn resolve_version(major: u8, minor: u8) -> MsgTypeResult<Self> {
                match major {
                    #resolve_fn_match
                    _ => Err(MsgTypeError::major_ver_err(major)),
                }
            }

            fn as_protocol_parts(&self) -> (&str, u8, u8) {
                let (major, minor) = match self {
                    #as_parts_fn_match
                };

                (Self::FAMILY, major, minor)
            }
        }
    };

    Ok(expanded)
}
