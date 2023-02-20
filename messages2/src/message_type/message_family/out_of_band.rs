use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use crate::error::{MsgTypeError, MsgTypeResult};

use super::{
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    MessageFamily,
};

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[semver(family = "out-of-band")]
pub enum OutOfBand {
    V1(OutOfBandV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(OutOfBand, MessageFamily)]
#[semver(major = 1)]
pub enum OutOfBandV1 {
    V1_1(OutOfBandV1_1),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveFrom, MessageType)]
#[transitive(OutOfBandV1, OutOfBand, MessageFamily)]
#[strum(serialize_all = "kebab-case")]
#[semver(minor = 1)]
pub enum OutOfBandV1_1 {
    Invitation,
    HandshakeReuse,
    HandshakeReuseAccepted,
}
