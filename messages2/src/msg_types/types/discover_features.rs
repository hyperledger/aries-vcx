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
#[semver(protocol = "discover-features")]
pub enum DiscoverFeatures {
    V1(DiscoverFeaturesV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(DiscoverFeatures, Protocol)))]
#[semver(major = 1, parent = "DiscoverFeatures", actors(Actor::Requester, Actor::Responder))]
pub enum DiscoverFeaturesV1 {
    V1_0(DiscoverFeaturesV1_0),
}

#[derive(Copy, Clone, Debug, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(DiscoverFeaturesV1, DiscoverFeatures, Protocol)))]
#[semver(minor = 1, parent = "DiscoverFeaturesV1")]
pub struct DiscoverFeaturesV1_0;

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, MessageType)]
#[strum(serialize_all = "kebab-case")]
#[semver(parent = "DiscoverFeaturesV1_0")]
pub enum DiscoverFeaturesV1_0Kind {
    Query,
    Disclose,
}
