use std::any::type_name;

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    aries_message::MsgWithType, composite_message::Message, message_type::MessageType, protocols::traits::MessageKind,
};

pub trait DelayedSerde: Sized {
    type MsgType: Into<MessageType>;

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

impl<C, D> DelayedSerde for Message<C, D>
where
    C: MessageKind,
    MessageType: From<<C as MessageKind>::Kind>,
    for<'a> MsgWithType<'a, Message<C, D>>: From<&'a Message<C, D>>,
    for<'d> Message<C, D>: Deserialize<'d>,
    Message<C, D>: Serialize,
{
    type MsgType = <C as MessageKind>::Kind;

    fn delayed_deserialize<'de, DE>(msg_type: Self::MsgType, deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: Deserializer<'de>,
    {
        let expected = Self::kind();
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
        MsgWithType::from(self).serialize(serializer)
    }
}
