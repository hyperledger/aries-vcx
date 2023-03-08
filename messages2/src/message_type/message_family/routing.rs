use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use crate::{
    error::{MsgTypeError, MsgTypeResult},
    message_type::MessageType,
};

use super::{
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    MessageFamily,
};

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(MessageFamily, MessageType))]
#[semver(family = "routing")]
pub enum Routing {
    V1(RoutingV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(Routing, MessageFamily, MessageType)))]
#[semver(major = 1, parent = "Routing")]
pub enum RoutingV1 {
    V1_0(RoutingV1_0),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(RoutingV1, Routing, MessageFamily, MessageType)))]
#[strum(serialize_all = "kebab-case")]
#[semver(minor = 0, parent = "RoutingV1")]
pub enum RoutingV1_0 {
    Forward,
}
