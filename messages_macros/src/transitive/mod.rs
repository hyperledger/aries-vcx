mod transitive_from;
mod transitive_try_from;

use syn::{spanned::Spanned, Attribute, Error, Meta, NestedMeta, Path, Result as SynResult, punctuated::Punctuated, Token};
pub use transitive_from::transitive_impl;
pub use transitive_try_from::transitive_try_from_impl;

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

/// Attributes kind used by transitive proc_macros.
enum TransitiveAttr {
    Transitive(Attribute),
    TransitiveAll(Attribute),
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
        .map(validate_param);

    // Ensure we were provided with at least two elements.
    let (first, last) = match (iter.next(), iter.next()) {
        (Some(first), Some(last)) => Ok((first?, last?)),
        _ => Err(Error::new(span, "at least two parameters needed")),
    }?;

    let output = MinimalAttrArgs { first, last, iter };

    Ok(output)
}

/// Checks if the attribute should be processed and maps it to [`TransitiveAttr`].
fn map_attr(attr: Attribute) -> Option<TransitiveAttr> {
    if attr.path.is_ident(TRANSITIVE) {
        Some(TransitiveAttr::Transitive(attr))
    } else if attr.path.is_ident(TRANSITIVE_ALL) {
        Some(TransitiveAttr::TransitiveAll(attr))
    } else {
        None
    }
}

/// Ensures we only accept types, not literals, integers or anything like that.
fn validate_param(param: NestedMeta) -> SynResult<Path> {
    match param {
        NestedMeta::Meta(Meta::Path(p)) => Ok(p),
        _ => Err(Error::new(param.span(), "only type paths accepted")),
    }
}
