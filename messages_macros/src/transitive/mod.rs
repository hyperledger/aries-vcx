mod transitive_from;
mod transitive_try_from;

use proc_macro2::{Ident, TokenStream};
use syn::{
    punctuated::Punctuated, spanned::Spanned, Attribute, DeriveInput, Error, Meta, NestedMeta, Path,
    Result as SynResult, Token,
};

pub use transitive_from::transitive_from_process_attr;
pub use transitive_try_from::transitive_try_from_process_attr;

const TRANSITIVE: &str = "transitive";
const TRANSITIVE_ALL: &str = "transitive_all";

/// Minimal arguments the transitive attributes should have,
/// along with an iterator of the remaining items
struct MinimalAttrArgs<I>
where
    I: Iterator<Item = SynResult<Path>>,
{
    first: Path,
    last: Path,
    iter: I,
}

/// Parse input and processes attributes with the provided closure.
pub fn transitive_impl<F>(input: DeriveInput, process_attr: F) -> SynResult<TokenStream>
where
    F: Fn(&Ident, Attribute) -> Option<SynResult<TokenStream>>,
{
    let name = input.ident;
    let mut expanded = TokenStream::new();
    for token_stream in input.attrs.into_iter().filter_map(|attr| process_attr(&name, attr)) {
        expanded.extend(token_stream?);
    }
    Ok(expanded)
}

/// Checks that the attribute was given the minimum needed arguments
/// and returns the arguments as a [`MinimalAttrArgs`] type.
fn validate_attr_args(attr: Attribute) -> SynResult<MinimalAttrArgs<impl Iterator<Item = SynResult<Path>>>> {
    // Save the span in case we issue errors.
    // Consuming the attribute arguments prevents us from doing that later.
    let span = attr.span();

    // Parse arguments and create an iterator of [`Path`] (types) items.
    let mut iter = attr
        .parse_args_with(Punctuated::<NestedMeta, Token![,]>::parse_terminated)?
        .into_iter()
        .map(is_type_path);

    // Ensure we were provided with at least two elements.
    let (first, last) = match (iter.next(), iter.next()) {
        (Some(first), Some(last)) => Ok((first?, last?)),
        _ => Err(Error::new(span, "at least two parameters needed")),
    }?;

    let output = MinimalAttrArgs { first, last, iter };

    Ok(output)
}

/// Ensures we only accept types, not literals, integers or anything like that.
fn is_type_path(param: NestedMeta) -> SynResult<Path> {
    match param {
        NestedMeta::Meta(Meta::Path(p)) => Ok(p),
        _ => Err(Error::new(param.span(), "only type paths accepted")),
    }
}
