use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use crate::{
    error::{MsgTypeError, MsgTypeResult},
    msg_types::actor::Actor,
    msg_types::registry::get_supported_version,
};

use super::{
    traits::{MajorVersion, MessageKind, MinorVersion, ProtocolName},
    Protocol,
};

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[semver(protocol = "out-of-band")]
pub enum OutOfBand {
    V1(OutOfBandV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(OutOfBand, Protocol)))]
#[semver(major = 1, parent = "OutOfBand", actors(Actor::Receiver, Actor::Sender))]
pub enum OutOfBandV1 {
    V1_1(OutOfBandV1_1),
}

#[derive(Copy, Clone, Debug, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(OutOfBandV1, OutOfBand, Protocol)))]
#[semver(minor = 1, parent = "OutOfBandV1")]
pub struct OutOfBandV1_1;

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, MessageType)]
#[strum(serialize_all = "kebab-case")]
#[semver(parent = "OutOfBandV1_1")]
pub enum OutOfBandV1_1Kind {
    Invitation,
    HandshakeReuse,
    HandshakeReuseAccepted,
}
