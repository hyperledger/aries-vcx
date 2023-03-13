use serde::{de::Error, Deserializer};

pub const MSG_TYPE: &str = "@type";

/// Used for creating a deserialization error.
/// Some messages, or rather, message types, are not meant
/// to be used as standalone messages.
///
/// E.g: Connection signature message type or credential preview message type.
pub fn not_standalone_msg<'de, D>(msg_type: &str) -> D::Error
where
    D: Deserializer<'de>,
{
    D::Error::custom(format!("{msg_type} is not a standalone message"))
}
