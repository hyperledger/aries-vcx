use derive_more::From;
use messages_macros::{MessageType, TransitiveFrom};
use strum_macros::{AsRefStr, EnumString};

use crate::error::{MsgTypeError, MsgTypeResult};

use super::{
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    MessageFamily,
};

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[semver(family = "basicmessage")]
pub enum BasicMessage {
    V1(BasicMessageV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(BasicMessage, MessageFamily)]
#[semver(major = 1)]
pub enum BasicMessageV1 {
    V1_0(BasicMessageV1_0),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveFrom, MessageType)]
#[transitive(BasicMessageV1, BasicMessage, MessageFamily)]
#[semver(minor = 0)]
#[strum(serialize_all = "kebab-case")]
pub enum BasicMessageV1_0 {
    Message,
}
