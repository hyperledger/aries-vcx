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
#[semver(family = "discover-features")]
pub enum DiscoverFeatures {
    V1(DiscoverFeaturesV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(DiscoverFeatures, MessageFamily, MessageType)))]
#[semver(major = 1, parent = "DiscoverFeatures")]
pub enum DiscoverFeaturesV1 {
    V1_0(DiscoverFeaturesV1_0),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(DiscoverFeaturesV1, DiscoverFeatures, MessageFamily, MessageType)))]
#[strum(serialize_all = "kebab-case")]
#[semver(minor = 1, parent = "DiscoverFeaturesV1")]
pub enum DiscoverFeaturesV1_0 {
    Query,
    Disclose,
}
