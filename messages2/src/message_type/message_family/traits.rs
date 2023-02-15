use std::str::FromStr;

use serde::{ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    error::{MsgTypeError, MsgTypeResult},
    message_type::MessageType,
};

const MSG_TYPE: &str = "@type";

pub trait ResolveMsgKind: Sized + FromStr + AsRef<str> {
    const MINOR: u8;

    fn resolve_kind(kind: &str) -> MsgTypeResult<Self> {
        kind.parse().map_err(|_| MsgTypeError::unknown_kind(kind.to_owned()))
    }

    fn as_minor_ver_parts(&self) -> (u8, &str) {
        (Self::MINOR, self.as_ref())
    }
}

pub trait ResolveMinorVersion: Sized {
    const MAJOR: u8;

    fn resolve_minor_ver(minor: u8, kind: &str) -> MsgTypeResult<Self>;

    fn as_full_ver_parts(&self) -> (u8, u8, &str);
}

pub trait ResolveMajorVersion: Sized {
    const FAMILY: &'static str;

    fn resolve_major_ver(major: u8, minor: u8, kind: &str) -> MsgTypeResult<Self>;

    fn as_msg_type_parts(&self) -> (&str, u8, u8, &str);
}

pub trait ConcreteMessage {
    type Kind: Into<MessageType> + PartialEq;

    fn kind() -> Self::Kind;
}

pub trait DelayedSerde: Sized {
    type Seg: Into<MessageType>;

    fn delayed_deserialize<'de, D>(seg: Self::Seg, deserializer: D) -> Result<Self, D::Error>
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
    type Seg = <Self as ConcreteMessage>::Kind;

    fn delayed_deserialize<'de, D>(seg: Self::Seg, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if seg == Self::kind() {
            Self::deserialize(deserializer)
        } else {
            todo!()
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
