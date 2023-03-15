use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::{
    traits::{MajorVersion, MinorVersion, ProtocolName},
    Protocol,
};
use crate::msg_types::{actor::Actor, registry::get_supported_version};

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
#[semver(minor = 0, parent = "DiscoverFeaturesV1")]
pub struct DiscoverFeaturesV1_0;

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, MessageType)]
#[strum(serialize_all = "kebab-case")]
#[semver(parent = "DiscoverFeaturesV1_0")]
pub enum DiscoverFeaturesV1_0Kind {
    Query,
    Disclose,
}

#[cfg(test)]
mod tests {
    use crate::misc::test_utils;

    use super::DiscoverFeaturesV1_0;

    const PROTOCOL: &str = "https://didcomm.org/discover-features/1.0";
    const VERSION_RESOLUTION_PROTOCOL: &str = "https://didcomm.org/discover-features/1.255";
    const UNSUPPORTED_VERSION_PROTOCOL: &str = "https://didcomm.org/discover-features/2.0";

    const KIND_QUERY: &str = "query";
    const KIND_DISCLOSE: &str = "disclose";

    #[test]
    fn test_protocol_discover_features() {
        test_utils::test_protocol(PROTOCOL, DiscoverFeaturesV1_0)
    }

    #[test]
    fn test_version_resolution_discover_features() {
        test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, DiscoverFeaturesV1_0)
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_discover_features() {
        test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, DiscoverFeaturesV1_0)
    }

    #[test]
    fn test_msg_type_query() {
        test_utils::test_msg_type(PROTOCOL, KIND_QUERY, DiscoverFeaturesV1_0)
    }

    #[test]
    fn test_msg_type_disclose() {
        test_utils::test_msg_type(PROTOCOL, KIND_DISCLOSE, DiscoverFeaturesV1_0)
    }
}
