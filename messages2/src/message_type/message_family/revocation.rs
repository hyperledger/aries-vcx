use derive_more::From;
use messages_macros::{MessageType, TransitiveFrom};
use strum_macros::{AsRefStr, EnumString};

use crate::error::{MsgTypeError, MsgTypeResult};

use super::{
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    MessageFamily,
};

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[semver(family = "revocation_notification")]
pub enum Revocation {
    V1(RevocationV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(Revocation, MessageFamily)]
#[semver(major = 1)]
pub enum RevocationV1 {
    V1_0(RevocationV1_0),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveFrom, MessageType)]
#[transitive(RevocationV1, Revocation, MessageFamily)]
#[strum(serialize_all = "kebab-case")]
#[semver(minor = 0)]
pub enum RevocationV1_0 {
    Revoke,
    Ack,
}
