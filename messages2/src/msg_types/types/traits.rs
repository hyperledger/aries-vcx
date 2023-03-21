use std::{marker::PhantomData, str::FromStr};

use crate::{error::MsgTypeResult, maybe_known::MaybeKnown, msg_types::actor::Actor};

pub trait MessageKind: FromStr + AsRef<str> {
    type Parent: MajorVersion;

    fn parent() -> Self::Parent;
}

pub trait MajorVersion: Sized {
    type Actors: IntoIterator<Item = MaybeKnown<Actor>>;

    const MAJOR: u8;

    fn try_resolve_version(minor: u8) -> MsgTypeResult<Self>;

    fn as_version_parts(&self) -> (u8, u8);

    fn actors(&self) -> Self::Actors;

    fn kind<T>(_: PhantomData<T>, kind: &str) -> crate::error::MsgTypeResult<T>
    where
        T: FromStr + MessageKind,
    {
        T::from_str(kind).map_err(|_| crate::error::MsgTypeError::unknown_kind(kind.to_owned()))
    }
}

pub trait ProtocolName: Sized {
    const PROTOCOL: &'static str;

    fn try_from_version_parts(major: u8, minor: u8) -> MsgTypeResult<Self>;

    fn as_protocol_parts(&self) -> (&'static str, u8, u8);
}
