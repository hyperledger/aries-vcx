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
#[semver(family = "connections")]
pub enum Connection {
    V1(ConnectionV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(Connection, MessageFamily)]
#[semver(major = 1)]
pub enum ConnectionV1 {
    V1_0(ConnectionV1_0),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveFrom, MessageType)]
#[transitive(ConnectionV1, Connection, MessageFamily)]
#[strum(serialize_all = "snake_case")]
#[semver(minor = 0)]
pub enum ConnectionV1_0 {
    Invitation,
    Request,
    Response,
    ProblemReport,
}
