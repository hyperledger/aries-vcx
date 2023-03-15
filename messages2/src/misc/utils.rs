use chrono::{DateTime, Utc};
use serde::{de::Error, Deserializer, Serialize};

pub const MSG_TYPE: &str = "@type";
pub const DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%SZ";

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

pub fn serialize_datetime<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    format_args!("{}", dt.format(DATETIME_FORMAT)).serialize(serializer)
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
