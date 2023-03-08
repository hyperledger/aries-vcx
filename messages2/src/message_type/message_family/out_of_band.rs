use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use crate::{
    error::{MsgTypeError, MsgTypeResult},
    message_type::actor::Actor,
    message_type::MessageType,
};

use super::{
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    MessageFamily,
};

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(MessageFamily, MessageType))]
#[semver(family = "out-of-band")]
pub enum OutOfBand {
    V1(OutOfBandV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(OutOfBand, MessageFamily, MessageType)))]
#[semver(major = 1, parent = "OutOfBand", actors(Actor::Receiver, Actor::Sender))]
pub enum OutOfBandV1 {
    V1_1(OutOfBandV1_1),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(OutOfBandV1, OutOfBand, MessageFamily, MessageType)))]
#[strum(serialize_all = "kebab-case")]
#[semver(minor = 1, parent = "OutOfBandV1")]
pub enum OutOfBandV1_1 {
    Invitation,
    HandshakeReuse,
    HandshakeReuseAccepted,
}
