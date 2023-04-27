use serde::{Deserializer, Serializer};

/// Trait used for postponing serialization/deserialization of a message.
///
/// It's main purpose is to allow us to navigate through the [`crate::msg_types::Protocol`]
/// and message kind to deduce which type we must deserialize to
/// or which [`crate::msg_types::Protocol`] + `message kind` we must construct
/// for the `@type` field of a particular message.
pub(crate) trait DelayedSerde: Sized {
    /// Long live GAT's!
    ///
    /// This allows us to pass a `&str` with
    /// a generic lifetime so that we can
    /// parse it later, when the message kind type
    /// can be determined.
    type MsgType<'a>;

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}
