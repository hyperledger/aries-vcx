//! Module that handles operations related solely to the protocol of a message, instead of it's content.
//! The main type, [`Protocol`], represents a protocol name along with its (both major and minor) version.
//!
//! The module contains other types that work adjacently to the [`Protocol`] to represent a message kind,
//! and along the protocol they make up the `@type` field of a message.

pub(crate) mod registry;
mod role;
pub mod traits;
pub mod types;

use std::{marker::PhantomData, str::FromStr};

use serde::{de::Error, Deserialize, Serialize};

pub use role::Role;
pub use types::{
    basic_message, connection, cred_issuance, discover_features, notification, out_of_band, present_proof,
    report_problem, revocation, routing, trust_ping, Protocol,
};

use self::traits::MessageKind;

/// Type used for deserialization of a fully qualified message type. After deserialization,
/// it is matched on to determine the actual message struct to deserialize to.
///
/// The [`Protocol`] and kind represent a complete `@type` field.
#[derive(Debug, PartialEq)]
pub(crate) struct MessageType<'a> {
    /// The [`Protocol`] part of the message type (e.g: https://didcomm.org/connections/1.0)
    pub protocol: Protocol,
    /// The message kind of the specific protocol (e.g: request)
    pub kind: &'a str,
}

impl<'de> Deserialize<'de> for MessageType<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize to &str
        let msg_type_str = <&str>::deserialize(deserializer)?;

        // Split (from the right) at the first '/'.
        // The first element will be the string repr of the protocol
        // while the second will be the message kind.
        let Some((protocol_str, kind)) = msg_type_str.rsplit_once('/') else {
            return Err(D::Error::custom(format!("Invalid message type: {msg_type_str}")));
        };

        // Parse the Protocol instance
        let protocol = match Protocol::from_str(protocol_str) {
            Ok(v) => Ok(v),
            Err(e) => {
                let msg = format!("Cannot parse message type: {msg_type_str}; Error: {e}");
                Err(D::Error::custom(msg))
            }
        }?;

        // Create instance to be passed for specialized message deserialization later.
        let msg_type = Self { protocol, kind };
        Ok(msg_type)
    }
}

/// Type used for serialization of a message along with appending it's `@type` field.
#[derive(Serialize)]
pub(crate) struct MsgWithType<'a, T, K>
where
    K: MessageKind,
    Protocol: From<K::Parent>,
{
    #[serde(rename = "@type")]
    #[serde(serialize_with = "serialize_msg_type")]
    kind: K,
    #[serde(flatten)]
    message: &'a T,
}

impl<'a, T, K> MsgWithType<'a, T, K>
where
    K: MessageKind,
    Protocol: From<K::Parent>,
{
    pub fn new(kind: K, message: &'a T) -> Self {
        Self { kind, message }
    }
}

/// Used for retrieving the [`Protocol`] first from the message kind
/// and serializing them together.
pub fn serialize_msg_type<S, K>(kind: &K, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    K: MessageKind,
    Protocol: From<K::Parent>,
{
    let kind = kind.as_ref();
    let protocol = Protocol::from(K::parent());

    format_args!("{protocol}/{kind}").serialize(serializer)
}

/// Type used for binding an impl of [`MessageKind`] to a variant of an enum implementing
/// [`crate::msg_types::traits::ProtocolVersion`].
///
/// The main reasons for abstracting over [`PhantomData`] is to make the generic type easier on the eyes
/// and, more importantly, to be able to define a method to convert from a `&str` to the bound type [`T`].
///
/// Technically, a trait implemented on [`PhantomData`] would've achieved the same thing, but would require
/// an import wherever it was used. A simple function accepting the [`PhantomData`] argument along with the `&str`
/// would've also worked, but it would be less ergonomic.
///
/// As per why the generic type is `fn() -> T` and not just `T`, the short story is *ownership*.
///
/// The long story is that `PhantomData<T>` tells the drop checker that we *own* `T`, which we
/// don't. While still a covariant, `fn() -> T` does not mean we own the `T`, so that lets the drop
/// checker be more permissive. Not really important for our current use case, but it is
/// *idiomatic*.
///
/// Good reads and references:
/// - https://doc.rust-lang.org/std/marker/struct.PhantomData.html
/// - https://doc.rust-lang.org/nomicon/phantom-data.html
/// - https://doc.rust-lang.org/nomicon/phantom-data.html
/// - https://doc.rust-lang.org/nomicon/dropck.html
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(transparent)]
pub struct MsgKindType<T: MessageKind>(PhantomData<fn() -> T>);

impl<T> MsgKindType<T>
where
    T: MessageKind,
{
    pub fn new() -> Self {
        Self(PhantomData)
    }

    /// Method used for taking a `&str` and trying to convert it
    /// into the bound type [`T`]. It uses [`FromStr`] under the hood.
    ///
    /// # Errors
    ///
    /// The method will error out if an instance of `T`
    /// could not be constructed from the `&str`.
    pub fn kind_from_str(&self, kind_str: &str) -> Result<T, <T as FromStr>::Err> {
        kind_str.parse()
    }
}
