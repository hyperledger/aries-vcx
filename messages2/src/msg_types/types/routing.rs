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
    #[msg_type(minor = 0, roles = "Role::Mediator")]
    V1_0(PhantomData<fn() -> RoutingV1_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum RoutingV1_0 {
    Forward,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_protocol_routing() {
        test_utils::test_serde(
            Protocol::from(RoutingV1::new_v1_0()),
            json!("https://didcomm.org/routing/1.0"),
        )
    }

    #[test]
    fn test_version_resolution_routing() {
        test_utils::test_msg_type_resolution("https://didcomm.org/routing/1.255", RoutingV1::new_v1_0())
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_routing() {
        test_utils::test_serde(
            Protocol::from(RoutingV1::new_v1_0()),
            json!("https://didcomm.org/routing/2.0"),
        )
    }

    #[test]
    fn test_msg_type_forward() {
        test_utils::test_msg_type("https://didcomm.org/routing/1.0", "forward", RoutingV1::new_v1_0())
    }
}
