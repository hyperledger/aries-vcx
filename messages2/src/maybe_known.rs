use serde::{Deserialize, Serialize};

/// Enum used to encapsulate `string-like` types which may
/// have variants we haven't implement yet.
///
/// Deserialization will be first attempted to the [`MaybeKnown::Known`] variant
/// and then, if that fails, to the [`MaybeKnown::Unknown`] variant.
///
/// E.g: a [`crate::msg_types::types::Protocol`] which might represent
/// a new protocol or simply a new major version.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum MaybeKnown<T> {
    Known(T),
    Unknown(String),
}
