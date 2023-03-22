use std::{any::type_name, fmt::Debug};

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use crate::msg_types::{traits::MessageKind, MsgWithType, Protocol};

/// Trait implemented for types that represent the protocol specific content of a
/// [`crate::message::Message`]. The trait links the type it's implemented on with the message kind
/// of the [`Protocol`] it's part of.
///
/// E.g: [`crate::protocols::trust_ping::PingResponseContent`] is linked to
/// [`crate::msg_types::types::trust_ping::TrustPingV1_0::PingResponse`], which in turn is linked to
/// [`crate::msg_types::types::trust_ping::TrustPingV1`].
pub trait MessageContent {
    type Kind;

    /// Returns an instance of the [`MessageContent::Kind`] type.
    fn kind() -> Self::Kind;
}

/// Trait used as trait bound for common [`DelayedSerde`] impls.
/// It is implemented for all the types that represent a complete message (apart from the `@type`
/// attribute).
///
/// It will pretty much always resort to the same functionality as [`MessageContent`], with the
/// caveat that this trait is instead implemented for the complete message instead of just the
/// content.
// Primarily exists because of [`crate::protocols::connection::invitation::Invitation`], which has
// multiple possible forms. See the docs for that type for more info. While unlikely, other types
// might have to manually implement this in the future.
pub trait MessageWithKind {
    type MsgKind;

    /// Returns an instance of the [`MessageWithKind::Kind`] type.
    fn msg_kind() -> Self::MsgKind;
}

/// Trait used for postponing serialization/deserialization of a message.
/// It's main purpose is to allow us to navigate through the [`Protocol`]
/// and message kind to deduce which type we must deserialize to
/// or which [`Protocol`] + `message kind` we must construct for the `@type` field
/// of a particular message.
pub trait DelayedSerde: Sized {
    type MsgType<'a>;

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

/// Blanket impl for serializing/deserializing end messages.
/// By end message one should understand the struct/enum representing the
/// message with the necessary fields as defined in a protocol RFC.
///
/// The versioning/protocol layers fall outside of this blanket impl and must implement
/// [`DelayedSerde`] manually.
impl<T> DelayedSerde for T
where
    T: MessageWithKind + Serialize,
    for<'a> T: Deserialize<'a>,
    T::MsgKind: MessageKind + AsRef<str> + PartialEq + Debug,
    Protocol: From<<T::MsgKind as MessageKind>::Parent>,
{
    type MsgType<'a> = T::MsgKind;

    fn delayed_deserialize<'de, DE>(msg_type: Self::MsgType<'de>, deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: Deserializer<'de>,
    {
        let expected = T::msg_kind();

        if msg_type == expected {
            Self::deserialize(deserializer)
        } else {
            let msg = format!(
                "Failed deserializing {}; Expected kind: {:?}, found: {:?}",
                type_name::<Self>(),
                expected,
                msg_type
            );
            Err(DE::Error::custom(msg))
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let kind = T::msg_kind();
        let kind = kind.as_ref();
        let protocol = Protocol::from(Self::MsgType::parent());

        MsgWithType::new(format_args!("{protocol}/{kind}"), self).serialize(serializer)
    }
}
