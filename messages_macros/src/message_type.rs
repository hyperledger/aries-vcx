use darling::{
    ast::{Data, Fields},
    FromDeriveInput, FromVariant,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated, spanned::Spanned, DeriveInput, Error, GenericArgument, Path, PathArguments, PathSegment,
    Result as SynResult, Token, Type, TypePath, ReturnType,
};

#[derive(FromDeriveInput)]
#[darling(attributes(msg_type), supports(enum_newtype))]
struct Protocol {
    ident: Ident,
    data: Data<MajorVerVariant, ()>,
    protocol: String,
}

#[derive(FromVariant)]
#[darling(attributes(msg_type))]
struct MajorVerVariant {
    ident: Ident,
    fields: Fields<Type>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(msg_type), supports(enum_newtype))]
struct Version {
    ident: Ident,
    data: Data<MinorVerVariant, ()>,
    major: u8,
}

#[derive(FromVariant)]
#[darling(attributes(msg_type))]
struct MinorVerVariant {
    ident: Ident,
    fields: Fields<Type>,
    minor: u8,
    actors: Punctuated<Path, Token![,]>,
}

pub fn message_type_impl(input: DeriveInput) -> SynResult<TokenStream> {
    if let Ok(protocol) = Protocol::from_derive_input(&input) {
        Ok(process_protocol(protocol))
    } else if let Ok(version) = Version::from_derive_input(&input) {
        process_version(version)
    } else {
        Err(Error::new(input.span(), "invalid arguments"))
    }
}

fn process_protocol(Protocol { ident, data, protocol }: Protocol) -> TokenStream {
    // The macro only accepts enums
    let Data::Enum(variants) = data else {unreachable!()};

    let mut try_from_match_arms = Vec::new();
    let mut as_parts_match_arms = Vec::new();
    let mut field_impls = Vec::new();

    for MajorVerVariant { ident, mut fields } in variants {
        // Newtype enums would always have one field
        let field = fields.fields.pop().expect("only implemented on newtype enums");

        try_from_match_arms.push(quote! {#field::MAJOR => #field::try_resolve_version(minor).map(Self::#ident)});
        as_parts_match_arms.push(quote! {Self::#ident(v) => v.as_version_parts()});
        field_impls.push(quote! {impl #field { const PROTOCOL: &str = #protocol; }});
    }

    quote! {
        impl crate::msg_types::types::traits::ProtocolName for #ident {
            const PROTOCOL: &'static str = #protocol;

            fn try_from_version_parts(major: u8, minor: u8) -> crate::error::MsgTypeResult<Self> {
                use crate::msg_types::types::traits::MajorVersion;

                match major {
                    #(#try_from_match_arms),*,
                    _ => Err(crate::error::MsgTypeError::major_ver_err(major)),
                }
            }

            fn as_protocol_parts(&self) -> (&'static str, u8, u8) {
                use crate::msg_types::types::traits::MajorVersion;

                let (major, minor) = match self {
                    #(#as_parts_match_arms),*,
                };

                (Self::PROTOCOL, major, minor)
            }
        }

        #(#field_impls)*
    }
}

fn process_version(Version { ident, data, major }: Version) -> SynResult<TokenStream> {
    // The macro only accepts enums
    let Data::Enum(variants) = data else {unreachable!()};

    let mut try_resolve_match_arms = Vec::new();
    let mut as_parts_match_arms = Vec::new();
    let mut actors_match_arms = Vec::new();
    let mut constructor_impls = Vec::new();
    let mut msg_kind_impls = Vec::new();

    for MinorVerVariant {
        ident: var_ident,
        minor,
        actors,
        mut fields,
    } in variants
    {
        // Newtype enums would always have one field
        let field = fields.fields.pop().expect("only implemented on newtype enums");
        let span = field.span();
        let actors = actors.into_iter();

        let path = match field {
            Type::Path(p) => Ok(p),
            _ => Err(Error::new(span, "expecting type path")),
        }?;

        let segment = first_path_segment(path)?;

        let PathArguments::AngleBracketed(args) = segment.arguments else { return Err(make_type_param_err(span)) };
        let arg = args.args.into_iter().next().ok_or_else(|| make_type_param_err(span))?;
        let GenericArgument::Type(Type::BareFn(fn_def)) = arg else { return Err(make_type_param_err(span)); };

        let ReturnType::Type(_, ty) = fn_def.output else { return Err(make_type_param_err(span)); };

        let constructor_fn_str = format!("new_{var_ident}").to_lowercase();
        let constructor_fn = Ident::new(&constructor_fn_str, var_ident.span());

        try_resolve_match_arms.push(quote! {#minor => Ok(Self::#var_ident(std::marker::PhantomData))});
        as_parts_match_arms.push(quote! {Self::#var_ident(_) => #minor});
        actors_match_arms
            .push(quote! {Self::#var_ident(_) => vec![#(crate::maybe_known::MaybeKnown::Known(#actors)),*]});
        constructor_impls.push(quote! {pub fn #constructor_fn() -> Self {Self::#var_ident(std::marker::PhantomData)}});
        msg_kind_impls.push(quote! {
            impl crate::msg_types::types::traits::MessageKind for #ty {
                type Parent = #ident;

                fn parent() -> Self::Parent {
                    #ident::#var_ident(std::marker::PhantomData)
                }
            }
        });
    }

    let expanded = quote! {
        impl crate::msg_types::types::traits::MajorVersion for #ident {
            type Actors = Vec<crate::maybe_known::MaybeKnown<crate::msg_types::role::Role>>;

            const MAJOR: u8 = #major;

            fn try_resolve_version(minor: u8) -> crate::error::MsgTypeResult<Self> {
                let protocol = Self::PROTOCOL;
                let major = Self::MAJOR;

                let Some(minor) = crate::msg_types::registry::get_supported_version(protocol, major, minor) else {
                    return Err(crate::error::MsgTypeError::minor_ver_err(minor));
                };

                match minor {
                    #(#try_resolve_match_arms),*,
                    _ => Err(crate::error::MsgTypeError::minor_ver_err(minor)),
                }
            }

            fn as_version_parts(&self) -> (u8, u8) {
                let minor = match self {
                    #(#as_parts_match_arms),*,
                };

                (Self::MAJOR, minor)
            }

            fn actors(&self) -> Self::Actors {
                match self {
                    #(#actors_match_arms),*,
                }
            }
        }

        impl #ident {
            #(#constructor_impls)*
        }

        #(#msg_kind_impls)*
    };

    Ok(expanded)
}

fn make_type_param_err(span: Span) -> Error {
    Error::new(span, "expecting a type parameter like PhantomData<T>")
}

fn first_path_segment(path: TypePath) -> SynResult<PathSegment> {
    let span = path.span();

    path.path
        .segments
        .into_iter()
        .next()
        .ok_or_else(|| make_type_param_err(span))
}
