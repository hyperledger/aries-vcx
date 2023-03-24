use darling::{
    ast::{Data, Fields},
    FromDeriveInput, FromVariant,
};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated, spanned::Spanned, DeriveInput, Error, GenericArgument, Path, PathArguments, PathSegment,
    Result as SynResult, Token, Type, TypePath,
};

/// Matches the input from deriving the macro
/// on a protocol enum.
///
/// E.g: `Routing`
#[derive(FromDeriveInput)]
#[darling(attributes(msg_type), supports(enum_newtype))]
struct Protocol {
    ident: Ident,
    data: Data<MajorVerVariant, ()>,
    protocol: String,
}

/// Matches the input of a major version variant of a protocol enum
/// that derives the macro.
///
/// E.g: the `RoutingV1` in `Routing::RoutingV1`
#[derive(FromVariant)]
#[darling(attributes(msg_type))]
struct MajorVerVariant {
    ident: Ident,
    fields: Fields<Type>,
}

/// Matches the input from deriving the macro on a
/// major version enum.
///
/// E.g: `RoutingV1`
#[derive(FromDeriveInput)]
#[darling(attributes(msg_type), supports(enum_newtype))]
struct Version {
    ident: Ident,
    data: Data<MinorVerVariant, ()>,
    major: u8,
}

/// Matches the input of a minor version variant of a major version enum
/// that derives the macro.
///
/// E.g: the `V1_0` in `RoutingV1::V1_0`
#[derive(FromVariant)]
#[darling(attributes(msg_type))]
struct MinorVerVariant {
    ident: Ident,
    fields: Fields<Type>,
    minor: u8,
    roles: Punctuated<Path, Token![,]>,
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

    // Storage for the try_from_version_parts() function match arms
    let mut try_from_match_arms = Vec::new();

    // Storage for the as_protocol_parts() function match arms
    let mut as_parts_match_arms = Vec::new();

    // Storage for the const PROTOCOL impls for the types encapsulated by the enum variants
    let mut field_impls = Vec::new();

    for MajorVerVariant { ident, fields } in variants {
        let field = extract_field_type(fields);

        // If the input u8 matches MAJOR, call the try_resolve_version() method of the encapsulated type.
        // Then wrap it in the enum this is derived on.
        try_from_match_arms.push(quote! {#field::MAJOR => #field::try_resolve_version(minor).map(Self::#ident)});

        // Match on the enum variant and call the as_version_parts() method.
        as_parts_match_arms.push(quote! {Self::#ident(v) => v.as_version_parts()});

        // Generate an impl with const PROTOCOL set the to the string literal passed in the macro attribute
        field_impls.push(quote! {impl #field { const PROTOCOL: &str = #protocol; }});
    }

    quote! {
        impl crate::msg_types::traits::ProtocolName for #ident {
            const PROTOCOL: &'static str = #protocol;

            fn try_from_version_parts(major: u8, minor: u8) -> crate::error::MsgTypeResult<Self> {
                use crate::msg_types::traits::ProtocolVersion;

                match major {
                    #(#try_from_match_arms),*,
                    _ => Err(crate::error::MsgTypeError::major_ver_err(major)),
                }
            }

            fn as_protocol_parts(&self) -> (&'static str, u8, u8) {
                use crate::msg_types::traits::ProtocolVersion;

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

    // Storage for the try_resolve_version() function match arms
    let mut try_resolve_match_arms = Vec::new();

    // Storage for the as_version_parts() function match arms
    let mut as_parts_match_arms = Vec::new();

    // Storage for the roles() function match arms
    let mut roles_match_arms = Vec::new();

    // Storage for the enum constructors that are based on variants' version
    let mut constructor_impls = Vec::new();

    // Storage for the MessageKind trait impls on the
    // types bound to the variant through PhantomData.
    let mut msg_kind_impls = Vec::new();

    for MinorVerVariant {
        ident: var_ident,
        minor,
        roles,
        fields,
    } in variants
    {
        // We need an iterator so we can wrap each role in MaybeKnown when destructuring.
        let roles = roles.into_iter();

        let field = extract_field_type(fields);
        let target_type = extract_field_target_type(field)?;
        let constructor_fn = make_constructor_fn(&var_ident);

        // If in the input u8 matches the minor version provided in the macro attribute
        // generate an instance of the enum with the variant in this iteration.
        try_resolve_match_arms.push(quote! {#minor => Ok(Self::#constructor_fn())});

        // If the variant matches the one in this iteration, return the minor version provided
        // in the macro attribute.
        as_parts_match_arms.push(quote! {Self::#var_ident(_) => #minor});

        // If the variant matches the one in this iteration, return a Vec
        // containing each provided `Role` wrapped in `MaybeKnown::Known`.
        roles_match_arms.push(quote! {Self::#var_ident(_) => vec![#(crate::maybe_known::MaybeKnown::Known(#roles)),*]});

        // Implement a function such as `new_v1_0` which returns the enum variant in this iteration.
        constructor_impls
            .push(quote! {pub fn #constructor_fn() -> Self {Self::#var_ident(crate::msg_types::MsgKindType::new())}});

        // Implement MessageKind for the target type bound to the enum variant in this iteration.
        msg_kind_impls.push(quote! {
            impl crate::msg_types::traits::MessageKind for #target_type {
                type Parent = #ident;

                fn parent() -> Self::Parent {
                    #ident::#constructor_fn()
                }
            }
        });
    }

    let expanded = quote! {
        impl crate::msg_types::traits::ProtocolVersion for #ident {
            type Roles = Vec<crate::maybe_known::MaybeKnown<crate::msg_types::Role>>;

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

            fn roles(&self) -> Self::Roles {
                match self {
                    #(#roles_match_arms),*,
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

/// Helper to generate an error in case the encapsulated type
/// in the enum variant is not as expected.
fn make_type_param_err(span: Span) -> Error {
    Error::new(span, "expecting a type parameter form: PhantomData<fn() -> T>")
}

/// Extracts the last (and only) field of the variant.
/// Newtype enums would always have one field, and the
/// macro is restricted to support just `enum_newtype`.
fn extract_field_type(mut fields: Fields<Type>) -> Type {
    fields.fields.pop().expect("only implemented on newtype enums")
}

/// The variant field type is of the form [`std::marker::PhantomData<fn() -> T>`].
/// We need to get the `T`.
fn extract_field_target_type(field: Type) -> SynResult<TypePath> {
    let mut span = field.span();

    // `PhantomData<_>` is a TypePath
    let Type::Path(path) = field else { return Err(make_type_param_err(span)) };

    // Getting the last, and most likely only, segment of the type path.
    let segment = last_path_segment(path)?;
    span = segment.span();

    // Extract the generics from their angle bracketed container.
    // E.g: <T, U, V> -> an iter returning T, U and V
    let PathArguments::AngleBracketed(args) = segment.arguments else { return Err(make_type_param_err(span)) };
    span = args.span();

    // This iterates over the generics provided.
    // We, again, expect just one, `fn() -> T`.
    let arg = args.args.into_iter().next().ok_or_else(|| make_type_param_err(span))?;
    span = arg.span();

    // We expect the generic to be a type, particularly a BareFn.
    let GenericArgument::Type(Type::Path(ty)) = arg else { return Err(make_type_param_err(span)); };

    // Return `T`
    Ok(ty)
}

/// Helper used to generate a `new_*` lowercase function name
/// based on the provided enum variant.
///
/// E.g: enum `A::V1_0` => variant `V1_0` => `new_v1_0`
fn make_constructor_fn(var_ident: &Ident) -> Ident {
    let constructor_fn_str = format!("new_{var_ident}").to_lowercase();
    Ident::new(&constructor_fn_str, var_ident.span())
}

/// Extracts the last segment of the type path.
/// This accommodates both situations like
/// - `PhantomData<_>`
/// - `std::marker::PhantomData<_>`
///
/// Making them both yield `PhantomData<_>`.
fn last_path_segment(path: TypePath) -> SynResult<PathSegment> {
    let span = path.span();

    path.path
        .segments
        .into_iter()
        .last()
        .ok_or_else(|| make_type_param_err(span))
}
