pub trait ConcreteMessage {
    type Kind;

    fn kind() -> Self::Kind;
}

use std::{any::type_name, fmt::Debug};

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use crate::msg_types::{types::traits::MessageKind, MsgWithType, Protocol};

pub trait HasKind {
    type KindType;

    fn kind_type() -> Self::KindType;
}

pub trait DelayedSerde: Sized {
    type MsgType<'a>;

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

impl<T> DelayedSerde for T
where
    T: HasKind + Serialize,
    for<'a> T: Deserialize<'a>,
    T::KindType: MessageKind + AsRef<str> + PartialEq + Debug,
    Protocol: From<<T::KindType as MessageKind>::Parent>,
{
    type MsgType<'a> = T::KindType;

    fn delayed_deserialize<'de, DE>(msg_type: Self::MsgType<'de>, deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: Deserializer<'de>,
    {
        let expected = T::kind_type();

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
        let kind = T::kind_type();
        let protocol = Protocol::from(Self::MsgType::parent());

        MsgWithType::new(format_args!("{protocol}/{}", kind.as_ref()), self).serialize(serializer)
    }
}
