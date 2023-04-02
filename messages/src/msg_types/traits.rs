use std::str::FromStr;

use crate::{error::MsgTypeResult, maybe_known::MaybeKnown, msg_types::role::Role};

/// Trait implemented on enums that represent the message kind of a protocol.
/// They link upstream to the [`ProtocolVersion`] impl enum they are part of.
/// The link downstream from the [`ProtocolVersion`] impl enum is done through the enum variant type binding.
///
/// E.g: `RoutingV1_0` would implement this, and its variants would look like `RoutingV1_0::Forward`.
/// From a protocol string such as `https://didcomm.org/routing/1.0/forward`, the variant would correspond to `forward`.
///
/// This trait is typically implemented through deriving [`messages_macros::MessageType`] on the [`ProtocolVersion`] impl enum
/// and annotating its variants.
pub trait MessageKind: FromStr + AsRef<str> {
    type Parent: ProtocolVersion;

    /// Returns an instance of the parent, which should be the correctly
    /// variant corresponding to the minor version this type represents message kinds for.
    fn parent() -> Self::Parent;
}

/// Trait implemented on enums that represent a major version of a protocol and where
/// the variants represent the minor version.
///
/// E.g: `RoutingTypeV1` would implement this, and its variants would look like `RoutingTypeV1::V1_0`.
/// From a protocol string such as `https://didcomm.org/routing/1.0/forward`, these would correspond to
/// the `1` and `0`, respectively.
///
/// This trait is typically implemented through deriving [`messages_macros::MessageType`].
pub trait ProtocolVersion: Sized {
    type Roles: IntoIterator<Item = MaybeKnown<Role>>;

    const MAJOR: u8;

    /// Tries to resolve the version of this protocol and returns
    /// an instance of itself on success.
    ///
    /// This is typically done by lookup in the [`crate::msg_types::registry::PROTOCOL_REGISTRY`]
    /// for the largest minor version less than or equal to the given `minor` argument.
    ///
    /// # Errors
    ///
    /// An error is returned if the version could not be resolved,
    /// which means we can't support the provided protocol version.
    //
    // NOTE: We already have the major version as a const declared in the trait.
    // so we just need the minor version.
    //
    // NOTE: The protocol name is also implemented at the enum level by the derive macro,
    // so it is accessible from here as well.
    fn try_resolve_version(minor: u8) -> MsgTypeResult<Self>;

    /// Returns the major and minor versions of the protocol.
    fn as_version_parts(&self) -> (u8, u8);

    /// Returns the roles the protocol provides.
    fn roles(&self) -> Self::Roles;
}

/// Trait implemented on enums that represent the name of a protocol.
///
/// E.g: `RoutingType` would implement this, and its variants would look like `RoutingType::V1`.
/// From a protocol string such as `https://didcomm.org/routing/1.0/forward`, the enum would correspond to `routing`.
///
/// This trait is typically implemented through deriving [`messages_macros::MessageType`] on the protocol specific enum.
pub trait ProtocolName: Sized {
    const PROTOCOL: &'static str;

    /// Tries to construct an instance of itself from the provided
    /// version parts.
    ///
    /// # Errors
    ///
    /// Will return an error if no variant matches the version parts provided.
    fn try_from_version_parts(major: u8, minor: u8) -> MsgTypeResult<Self>;

    /// Returns the protocol name, major and minor versions of the protocol.
    fn as_protocol_parts(&self) -> (&'static str, u8, u8);
}
