use derive_more::From;
use messages_macros::{MessageType, TransitiveFrom};
use strum_macros::{AsRefStr, EnumString};

use crate::error::{MsgTypeError, MsgTypeResult};

use super::{
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    MessageFamily,
};

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[semver(family = "routing")]
pub enum Routing {
    V1(RoutingV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(Routing, MessageFamily)]
#[semver(major = 1)]
pub enum RoutingV1 {
    V1_0(RoutingV1_0),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveFrom, MessageType)]
#[transitive(RoutingV1, Routing, MessageFamily)]
#[strum(serialize_all = "kebab-case")]
#[semver(minor = 0)]
pub enum RoutingV1_0 {
    Forward,
}
