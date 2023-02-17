use std::any::type_name;

use serde::{de::Error, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};

use crate::{aries_message::MSG_TYPE, message_type::MessageType, protocols::traits::ConcreteMessage};

pub trait DelayedSerde: Sized {
    type MsgType: Into<MessageType>;

    fn delayed_deserialize<'de, D>(seg: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;

    fn delayed_serialize<'a, M, F, S>(&self, state: &'a mut M, closure: &mut F) -> Result<S::Ok, S::Error>
    where
        M: SerializeMap,
        F: FnMut(&'a mut M) -> S,
        S: Serializer,
        S::Error: From<M::Error>;
}

impl<T> DelayedSerde for T
where
    for<'d> T: ConcreteMessage + Serialize + Deserialize<'d>,
{
    type MsgType = <Self as ConcreteMessage>::Kind;

    fn delayed_deserialize<'de, D>(seg: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expected = Self::kind();
        if seg == expected {
            Self::deserialize(deserializer)
        } else {
            let msg = format!(
                "Failed deserializing {}; Expected kind: {:?}, found: {:?}",
                type_name::<T>(),
                expected,
                seg
            );
            Err(D::Error::custom(msg))
        }
    }

    fn delayed_serialize<'a, M, F, S>(&self, state: &'a mut M, closure: &mut F) -> Result<S::Ok, S::Error>
    where
        M: SerializeMap,
        F: FnMut(&'a mut M) -> S,
        S: Serializer,
        S::Error: From<M::Error>,
    {
        let msg_type: MessageType = Self::kind().into();
        state.serialize_entry(MSG_TYPE, &msg_type)?;
        let serializer = closure(state);
        self.serialize(serializer)
    }
}
