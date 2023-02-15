use derive_more::From;
use strum_macros::{AsRefStr, EnumString};

use crate::{
    error::{MsgTypeError, MsgTypeResult},
    macros::transient_from,
};

use super::{
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    MessageFamily,
};

#[derive(Copy, Clone, Debug, From, PartialEq)]
pub enum OutOfBand {
    V1(OutOfBandV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq)]
pub enum OutOfBandV1 {
    V1_1(OutOfBandV1_1),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum OutOfBandV1_1 {
    Invitation,
    HandshakeReuse,
    HandshakeReuseAccepted,
}

transient_from!(OutOfBandV1_1, OutOfBandV1, OutOfBand, MessageFamily);

impl ResolveMsgKind for OutOfBandV1_1 {
    const MINOR: u8 = 1;
}

impl ResolveMinorVersion for OutOfBandV1 {
    const MAJOR: u8 = 1;

    fn resolve_minor_ver(minor: u8, kind: &str) -> MsgTypeResult<Self> {
        match minor {
            OutOfBandV1_1::MINOR => Ok(Self::V1_1(OutOfBandV1_1::resolve_kind(kind)?)),
            _ => Err(MsgTypeError::minor_ver_err(minor)),
        }
    }

    fn as_full_ver_parts(&self) -> (u8, u8, &str) {
        let (minor, kind) = match self {
            Self::V1_1(v) => v.as_minor_ver_parts(),
        };
        (Self::MAJOR, minor, kind)
    }
}

impl ResolveMajorVersion for OutOfBand {
    const FAMILY: &'static str = "out-of-band";

    fn resolve_major_ver(major: u8, minor: u8, kind: &str) -> MsgTypeResult<Self> {
        match major {
            OutOfBandV1::MAJOR => Ok(Self::V1(OutOfBandV1::resolve_minor_ver(minor, kind)?)),
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
