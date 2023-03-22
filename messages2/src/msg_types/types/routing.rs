use std::marker::PhantomData;

use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::Protocol;
use crate::msg_types::role::Role;

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[msg_type(protocol = "routing")]
pub enum Routing {
    V1(RoutingV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(Routing, Protocol))]
#[msg_type(major = 1)]
pub enum RoutingV1 {
    #[msg_type(minor = 0, actors = "Role::Mediator")]
    V1_0(PhantomData<fn() -> RoutingV1_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum RoutingV1_0 {
    Forward,
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::RoutingV1;
    use crate::misc::test_utils;

    const PROTOCOL: &str = "https://didcomm.org/routing/1.0";
    const VERSION_RESOLUTION_PROTOCOL: &str = "https://didcomm.org/routing/1.255";
    const UNSUPPORTED_VERSION_PROTOCOL: &str = "https://didcomm.org/routing/2.0";

    const KIND_FORWARD: &str = "forward";

    #[test]
    fn test_protocol_routing() {
        test_utils::test_protocol(PROTOCOL, RoutingV1::V1_0(PhantomData))
    }

    #[test]
    fn test_version_resolution_routing() {
        test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, RoutingV1::V1_0(PhantomData))
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_routing() {
        test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, RoutingV1::V1_0(PhantomData))
    }

    #[test]
    fn test_msg_type_forward() {
        test_utils::test_msg_type(PROTOCOL, KIND_FORWARD, RoutingV1::V1_0(PhantomData))
    }
}
