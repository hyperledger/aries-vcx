use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use crate::{
    error::{MsgTypeError, MsgTypeResult},
    message_type::actor::Actor,
    message_type::registry::get_supported_version,
};

use super::{
    traits::{MajorVersion, MessageKind, MinorVersion, ProtocolName},
    Protocol,
};

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[semver(family = "revocation_notification")]
pub enum Revocation {
    V2(RevocationV2),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(Revocation, Protocol)))]
#[semver(major = 2, parent = "Revocation", actors(Actor::Holder, Actor::Issuer))]
pub enum RevocationV2 {
    V2_0(RevocationV2_0),
}

#[derive(Copy, Clone, Debug, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(RevocationV2, Revocation, Protocol)))]
#[semver(minor = 0, parent = "RevocationV2")]
pub struct RevocationV2_0;

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, MessageType)]
#[strum(serialize_all = "kebab-case")]
#[semver(parent = "RevocationV2_0")]
pub enum RevocationV2_0Kind {
    Revoke,
    Ack,
}
