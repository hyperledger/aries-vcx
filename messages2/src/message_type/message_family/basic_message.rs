use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use crate::{
    error::{MsgTypeError, MsgTypeResult},
    message_type::actor::Actor,
    message_type::{registry::get_supported_version, MessageType},
};

use super::{
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    MessageFamily,
};

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(MessageFamily, MessageType))]
#[semver(family = "basicmessage")]
pub enum BasicMessage {
    V1(BasicMessageV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(BasicMessage, MessageFamily, MessageType)))]
#[semver(major = 1, parent = "BasicMessage", actors(Actor::Receiver, Actor::Sender))]
pub enum BasicMessageV1 {
    V1_0(BasicMessageV1_0),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(BasicMessageV1, BasicMessage, MessageFamily, MessageType)))]
#[semver(minor = 0, parent = "BasicMessageV1")]
#[strum(serialize_all = "kebab-case")]
pub enum BasicMessageV1_0 {
    Message,
}
