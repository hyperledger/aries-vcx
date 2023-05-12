use serde::{Deserialize, Serialize};

/// Enum used to encapsulate `string-like` types which may
/// have variants we haven't implement yet.
///
/// Deserialization will be first attempted to the [`MaybeKnown::Known`] variant
/// and then, if that fails, to the [`MaybeKnown::Unknown`] variant.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum MaybeKnown<T, U = String> {
    Known(T),
    Unknown(U),
}
