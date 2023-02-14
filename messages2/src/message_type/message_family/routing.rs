use derive_more::From;
use strum_macros::{AsRefStr, EnumString};

use crate::{error::{MsgTypeError, MsgTypeResult}, macros::transient_from};

use super::{traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind}, MessageFamily};

#[derive(From, PartialEq)]
pub enum Routing {
    V1(RoutingV1),
}

#[derive(From, PartialEq)]
pub enum RoutingV1 {
    V1_0(RoutingV1_0),
}

#[derive(AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum RoutingV1_0 {
    Forward,
}

transient_from!(RoutingV1_0, RoutingV1, Routing, MessageFamily);

impl ResolveMsgKind for RoutingV1_0 {
    const MINOR: u8 = 0;
}

impl ResolveMinorVersion for RoutingV1 {
    const MAJOR: u8 = 1;

    fn resolve_minor_ver(minor: u8, kind: &str) -> MsgTypeResult<Self> {
        match minor {
            RoutingV1_0::MINOR => Ok(Self::V1_0(RoutingV1_0::resolve_kind(kind)?)),
            _ => Err(MsgTypeError::minor_ver_err(minor)),
        }
    }

    fn as_full_ver_parts(&self) -> (u8, u8, &str) {
        let (minor, kind) = match self {
            Self::V1_0(v) => v.as_minor_ver_parts(),
        };
        (Self::MAJOR, minor, kind)
    }
}

impl ResolveMajorVersion for Routing {
    const FAMILY: &'static str = "routing";

    fn resolve_major_ver(major: u8, minor: u8, kind: &str) -> MsgTypeResult<Self> {
        match major {
            RoutingV1::MAJOR => Ok(Self::V1(RoutingV1::resolve_minor_ver(minor, kind)?)),
            _ => Err(MsgTypeError::major_ver_err(major)),
        }
    }

    fn as_msg_type_parts(&self) -> (&str, u8, u8, &str) {
        let (major, minor, kind) = match self {
            Self::V1(v) => v.as_full_ver_parts(),
        };

        (Self::FAMILY, major, minor, kind)
    }
}
