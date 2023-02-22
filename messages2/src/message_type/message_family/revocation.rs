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
#[semver(family = "revocation_notification")]
pub enum Revocation {
    V1(RevocationV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveInto, MessageType)]
#[transitive(all(Revocation, MessageFamily, MessageType))]
#[semver(major = 1)]
pub enum RevocationV1 {
    V1_0(RevocationV1_0),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveInto, MessageType)]
#[transitive(all(RevocationV1, Revocation, MessageFamily, MessageType))]
#[strum(serialize_all = "kebab-case")]
#[semver(minor = 0)]
pub enum RevocationV1_0 {
    Revoke,
    Ack,
}
