use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveInto;

use crate::{
    error::{MsgTypeError, MsgTypeResult},
    message_type::MessageType,
};

use super::{
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    MessageFamily,
};

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveInto, MessageType)]
#[transitive(MessageFamily, MessageType)]
#[semver(family = "basicmessage")]
pub enum BasicMessage {
    V1(BasicMessageV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveInto, MessageType)]
#[transitive(all(BasicMessage, MessageFamily, MessageType))]
#[semver(major = 1)]
pub enum BasicMessageV1 {
    V1_0(BasicMessageV1_0),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveInto, MessageType)]
#[transitive(all(BasicMessageV1, BasicMessage, MessageFamily, MessageType))]
#[semver(minor = 0)]
#[strum(serialize_all = "kebab-case")]
pub enum BasicMessageV1_0 {
    Message,
}
