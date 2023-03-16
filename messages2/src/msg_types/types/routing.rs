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
#[semver(protocol = "routing")]
pub enum Routing {
    V1(RoutingV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(Routing, Protocol)))]
#[semver(major = 1, parent = "Routing", actors(Actor::Mediator))]
pub enum RoutingV1 {
    V1_0(RoutingV1_0),
}

#[derive(Copy, Clone, Debug, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(RoutingV1, Routing, Protocol)))]
#[semver(minor = 0, parent = "RoutingV1")]
pub struct RoutingV1_0;

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, MessageType)]
#[strum(serialize_all = "kebab-case")]
#[semver(parent = "RoutingV1_0")]
pub enum RoutingV1_0Kind {
    Forward,
}

#[cfg(test)]
mod tests {
    use super::RoutingV1_0;
    use crate::misc::test_utils;

    const PROTOCOL: &str = "https://didcomm.org/routing/1.0";
    const VERSION_RESOLUTION_PROTOCOL: &str = "https://didcomm.org/routing/1.255";
    const UNSUPPORTED_VERSION_PROTOCOL: &str = "https://didcomm.org/routing/2.0";

    const KIND_FORWARD: &str = "forward";

    #[test]
    fn test_protocol_routing() {
        test_utils::test_protocol(PROTOCOL, RoutingV1_0)
    }

    #[test]
    fn test_version_resolution_routing() {
        test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, RoutingV1_0)
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_routing() {
        test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, RoutingV1_0)
    }

    #[test]
    fn test_msg_type_forward() {
        test_utils::test_msg_type(PROTOCOL, KIND_FORWARD, RoutingV1_0)
    }
}
