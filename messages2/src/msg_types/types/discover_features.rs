use std::marker::PhantomData;

use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::Protocol;
use crate::msg_types::actor::Actor;

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[msg_type(protocol = "discover-features")]
pub enum DiscoverFeatures {
    V1(DiscoverFeaturesV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(DiscoverFeatures, Protocol)))]
#[msg_type(major = 1)]
pub enum DiscoverFeaturesV1 {
    #[msg_type(minor = 0, actors = "Actor::Requester, Actor::Responder")]
    V1_0(PhantomData<DiscoverFeaturesV1_0Kind>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum DiscoverFeaturesV1_0Kind {
    Query,
    Disclose,
}

// #[cfg(test)]
// mod tests {
//     use super::DiscoverFeaturesV1_0;
//     use crate::misc::test_utils;

//     const PROTOCOL: &str = "https://didcomm.org/discover-features/1.0";
//     const VERSION_RESOLUTION_PROTOCOL: &str = "https://didcomm.org/discover-features/1.255";
//     const UNSUPPORTED_VERSION_PROTOCOL: &str = "https://didcomm.org/discover-features/2.0";

//     const KIND_QUERY: &str = "query";
//     const KIND_DISCLOSE: &str = "disclose";

//     #[test]
//     fn test_protocol_discover_features() {
//         test_utils::test_protocol(PROTOCOL, DiscoverFeaturesV1_0)
//     }

//     #[test]
//     fn test_version_resolution_discover_features() {
//         test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, DiscoverFeaturesV1_0)
//     }

//     #[test]
//     #[should_panic]
//     fn test_unsupported_version_discover_features() {
//         test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, DiscoverFeaturesV1_0)
//     }

//     #[test]
//     fn test_msg_type_query() {
//         test_utils::test_msg_type(PROTOCOL, KIND_QUERY, DiscoverFeaturesV1_0)
//     }

//     #[test]
//     fn test_msg_type_disclose() {
//         test_utils::test_msg_type(PROTOCOL, KIND_DISCLOSE, DiscoverFeaturesV1_0)
//     }
// }
