use derive_more::From;
use messages_macros::{MessageType, TransitiveFrom};
use strum_macros::{AsRefStr, EnumString};

use crate::error::{MsgTypeError, MsgTypeResult};

use super::{
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    MessageFamily,
};

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[semver(family = "discover-features")]
pub enum DiscoverFeatures {
    V1(DiscoverFeaturesV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(DiscoverFeatures, MessageFamily)]
#[semver(major = 1)]
pub enum DiscoverFeaturesV1 {
    V1_0(DiscoverFeaturesV1_0),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveFrom, MessageType)]
#[transitive(DiscoverFeaturesV1, DiscoverFeatures, MessageFamily)]
#[strum(serialize_all = "kebab-case")]
#[semver(minor = 1)]
pub enum DiscoverFeaturesV1_0 {
    Query,
    Disclose,
}
