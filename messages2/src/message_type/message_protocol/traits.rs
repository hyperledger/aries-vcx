use std::str::FromStr;

use crate::{error::MsgTypeResult, message_type::actor::Actor};

pub trait MessageKind: FromStr + AsRef<str> {
    type Parent: MinorVersion;

    fn parent() -> Self::Parent;
}

pub trait MinorVersion: Sized {
    type Parent: MajorVersion;

    const MINOR: u8;

    fn as_minor_version(&self) -> u8 {
        Self::MINOR
    }
}

pub trait MajorVersion: Sized {
    type Actors: IntoIterator<Item = Actor>;
    type Parent: ProtocolName;

    const MAJOR: u8;

    fn resolve_minor_ver(minor: u8) -> MsgTypeResult<Self>;

    fn as_version_parts(&self) -> (u8, u8);

    fn actors() -> Self::Actors;
}

pub trait ProtocolName: Sized {
    const FAMILY: &'static str;

    fn resolve_version(major: u8, minor: u8) -> MsgTypeResult<Self>;

    fn as_protocol_parts(&self) -> (&str, u8, u8);
}
