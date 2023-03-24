use chrono::{DateTime, Utc};
use serde::{de::Error, Deserializer, Serialize};

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

pub fn serialize_datetime<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use chrono::format::Fixed;
    use chrono::format::Item;
    use chrono::format::Numeric::*;
    use chrono::format::Pad::Zero;

    const FMT_ITEMS: &[Item<'static>] = &[
        Item::Numeric(Year, Zero),
        Item::Literal("-"),
        Item::Numeric(Month, Zero),
        Item::Literal("-"),
        Item::Numeric(Day, Zero),
        Item::Literal("T"),
        Item::Numeric(Hour, Zero),
        Item::Literal(":"),
        Item::Numeric(Minute, Zero),
        Item::Literal(":"),
        Item::Numeric(Second, Zero),
        Item::Fixed(Fixed::Nanosecond3),
        Item::Fixed(Fixed::TimezoneOffsetColonZ),
    ];

    format_args!("{}", dt.format_with_items(FMT_ITEMS.iter())).serialize(serializer)
}

pub fn serialize_opt_datetime<S>(dt: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match dt {
        Some(dt) => serialize_datetime(dt, serializer),
        None => serializer.serialize_none(),
    }
}

macro_rules! transit_to_aries_msg {
    ($content:ty, $($interm:ty),+) => {
        transit_to_aries_msg!($content:$crate::misc::NoDecorators, $($interm),+);
    };

    ($content:ty: $decorators:ty, $($interm:ty),+) => {
        impl From<$crate::msg_parts::MsgParts<$content, $decorators>> for $crate::AriesMessage {
            fn from(value: $crate::msg_parts::MsgParts<$content, $decorators>) -> Self {
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

macro_rules! into_msg_with_type {
    ($msg:ident, $kind:ident, $kind_var:ident) => {
        impl<'a> From<&'a $msg> for $crate::msg_types::MsgWithType<'a, $msg, $kind> {
            fn from(value: &'a $msg) -> $crate::msg_types::MsgWithType<'a, $msg, $kind> {
                $crate::msg_types::MsgWithType::new($kind::$kind_var, value)
            }
        }
    };
}

pub(crate) use generate_from_stmt;
pub(crate) use into_msg_with_type;
pub(crate) use transit_to_aries_msg;
