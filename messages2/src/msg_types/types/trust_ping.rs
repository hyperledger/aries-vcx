use std::marker::PhantomData;

use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::Protocol;
use crate::msg_types::role::Role;

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[msg_type(protocol = "trust_ping")]
pub enum TrustPing {
    V1(TrustPingV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(TrustPing, Protocol))]
#[msg_type(major = 1)]
pub enum TrustPingV1 {
    #[msg_type(minor = 0, roles = "Role::Sender, Role::Receiver")]
    V1_0(PhantomData<fn() -> TrustPingV1_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum TrustPingV1_0 {
    Ping,
    #[strum(serialize = "ping_response")]
    PingResponse,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_protocol_trust_ping() {
        test_utils::test_serde(
            Protocol::from(TrustPingV1::new_v1_0()),
            json!("https://didcomm.org/trust_ping/1.0"),
        )
    }

    #[test]
    fn test_version_resolution_trust_ping() {
        test_utils::test_msg_type_resolution("https://didcomm.org/trust_ping/1.255", TrustPingV1::new_v1_0())
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_trust_ping() {
        test_utils::test_serde(
            Protocol::from(TrustPingV1::new_v1_0()),
            json!("https://didcomm.org/trust_ping/2.0"),
        )
    }

    #[test]
    fn test_msg_type_ping() {
        test_utils::test_msg_type("https://didcomm.org/trust_ping/1.0", "ping", TrustPingV1::new_v1_0())
    }

    #[test]
    fn test_msg_type_ping_response() {
        test_utils::test_msg_type(
            "https://didcomm.org/trust_ping/1.0",
            "ping_response",
            TrustPingV1::new_v1_0(),
        )
    }
}
