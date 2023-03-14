use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use crate::{msg_types::actor::Actor, msg_types::registry::get_supported_version};

use super::{
    traits::{MajorVersion, MinorVersion, ProtocolName},
    Protocol,
};

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[semver(protocol = "basicmessage")]
pub enum BasicMessage {
    V1(BasicMessageV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(BasicMessage, Protocol)))]
#[semver(major = 1, parent = "BasicMessage", actors(Actor::Receiver, Actor::Sender))]
pub enum BasicMessageV1 {
    V1_0(BasicMessageV1_0),
}

#[derive(Copy, Clone, Debug, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(BasicMessageV1, BasicMessage, Protocol)))]
#[semver(minor = 0, parent = "BasicMessageV1")]
pub struct BasicMessageV1_0;

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, MessageType)]
#[strum(serialize_all = "kebab-case")]
#[semver(parent = "BasicMessageV1_0")]
pub enum BasicMessageV1_0Kind {
    Message,
}
