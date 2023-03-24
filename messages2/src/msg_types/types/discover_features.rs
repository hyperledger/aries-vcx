use std::marker::PhantomData;

use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::Protocol;
use crate::msg_types::role::Role;

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[msg_type(protocol = "discover-features")]
pub enum DiscoverFeaturesProtocol {
    V1(DiscoverFeaturesProtocolV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(DiscoverFeaturesProtocol, Protocol))]
#[msg_type(major = 1)]
pub enum DiscoverFeaturesProtocolV1 {
    #[msg_type(minor = 0, roles = "Role::Requester, Role::Responder")]
    V1_0(PhantomData<fn() -> DiscoverFeaturesProtocolV1_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum DiscoverFeaturesProtocolV1_0 {
    Query,
    Disclose,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_protocol_discover_features() {
        test_utils::test_serde(
            Protocol::from(DiscoverFeaturesProtocolV1::new_v1_0()),
            json!("https://didcomm.org/discover-features/1.0"),
        )
    }

    #[test]
    fn test_version_resolution_discover_features() {
        test_utils::test_msg_type_resolution(
            "https://didcomm.org/discover-features/1.255",
            DiscoverFeaturesProtocolV1::new_v1_0(),
        )
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_discover_features() {
        test_utils::test_serde(
            Protocol::from(DiscoverFeaturesProtocolV1::new_v1_0()),
            json!("https://didcomm.org/discover-features/2.0"),
        )
    }

    #[test]
    fn test_msg_type_query() {
        test_utils::test_msg_type(
            "https://didcomm.org/discover-features/1.0",
            "query",
            DiscoverFeaturesProtocolV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_disclose() {
        test_utils::test_msg_type(
            "https://didcomm.org/discover-features/1.0",
            "disclose",
            DiscoverFeaturesProtocolV1::new_v1_0(),
        )
    }
}
