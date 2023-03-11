use std::{
    any::type_name,
    fmt::{Arguments, Debug},
};

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    composite_message::Message,
    message_type::{message_protocol::traits::MessageKind, MessageFamily},
    protocols::traits::ConcreteMessage,
};

pub trait DelayedSerde: Sized {
    type MsgType<'a>;

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer;
}

impl<C, D> DelayedSerde for Message<C, D>
where
    C: ConcreteMessage,
    C::Kind: MessageKind + AsRef<str> + PartialEq + Debug,
    MessageFamily: From<<C::Kind as MessageKind>::Parent>,
    for<'d> Message<C, D>: Deserialize<'d>,
    Message<C, D>: Serialize,
{
    type MsgType<'a> = <C as ConcreteMessage>::Kind;

    fn delayed_deserialize<'de, DE>(msg_type: Self::MsgType<'de>, deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: Deserializer<'de>,
    {
        let expected = C::kind();

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
        #[derive(Serialize)]
        struct MsgWithType<'a, T> {
            #[serde(rename = "@type")]
            msg_type: Arguments<'a>,
            #[serde(flatten)]
            message: &'a T,
        }

        impl<'a, T> MsgWithType<'a, T> {
            fn new(msg_type: Arguments<'a>, message: &'a T) -> Self {
                Self { msg_type, message }
            }
        }

        let kind = Self::kind();
        let protocol = MessageFamily::from(Self::MsgType::parent());

        MsgWithType::new(format_args!("{protocol}/{}", kind.as_ref()), self).serialize(serializer)
    }
}
