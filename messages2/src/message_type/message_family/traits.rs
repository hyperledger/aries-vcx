use std::str::FromStr;

use crate::{
    error::{MsgTypeError, MsgTypeResult},
    message_type::actor::Actor,
};

pub trait ResolveMsgKind: Sized + FromStr + AsRef<str> {
    type Parent: ResolveMinorVersion;

    const MINOR: u8;

    fn resolve_kind(kind: &str) -> MsgTypeResult<Self> {
        kind.parse().map_err(|_| MsgTypeError::unknown_kind(kind.to_owned()))
    }

    fn as_minor_ver_parts(&self) -> (u8, &str) {
        (Self::MINOR, self.as_ref())
    }
}

pub trait ResolveMinorVersion: Sized {
    type Actors: IntoIterator<Item = Actor>;
    type Parent: ResolveMajorVersion;

    const MAJOR: u8;

    fn resolve_minor_ver(minor: u8, kind: &str) -> MsgTypeResult<Self>;

    fn as_full_ver_parts(&self) -> (u8, u8, &str);

    fn actors() -> Self::Actors;
}

pub trait ResolveMajorVersion: Sized {
    const FAMILY: &'static str;

    fn resolve_major_ver(major: u8, minor: u8, kind: &str) -> MsgTypeResult<Self>;

    fn as_msg_type_parts(&self) -> (&str, u8, u8, &str);
}
