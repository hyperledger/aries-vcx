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

macro_rules! transit_to_aries_msg {
    ($content:ty, $($interm:ty),+) => {
        transit_to_aries_msg!($content:$crate::misc::nothing::Nothing, $($interm),+);
    };

    ($content:ty: $decorators:ty, $($interm:ty),+) => {
        impl From<$crate::Message<$content, $decorators>> for $crate::AriesMessage {
            fn from(value: $crate::Message<$content, $decorators>) -> Self {
                Self::from($crate::misc::utils::generate_from_stmt!(value, $($interm),+))
            }
        }
    };
}

macro_rules! generate_from_stmt {
    ($val:expr, $interm:ty) => {
        <$interm>::from($val)
    };
    ($val:expr, $interm:ty, $($i:ty),+) => {
        $crate::misc::utils::generate_from_stmt!($crate::misc::utils::generate_from_stmt!($val, $interm), $($i),+)
    };
}

pub(crate) use generate_from_stmt;
pub(crate) use transit_to_aries_msg;
