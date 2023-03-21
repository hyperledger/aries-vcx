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
    #[msg_type(minor = 0, actors = "Role::Sender, Role::Receiver")]
    V1_0(PhantomData<TrustPingV1_0Kind>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum TrustPingV1_0Kind {
    Ping,
    #[strum(serialize = "ping_response")]
    PingResponse,
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::TrustPingV1;
    use crate::misc::test_utils;

    const PROTOCOL: &str = "https://didcomm.org/trust_ping/1.0";
    const VERSION_RESOLUTION_PROTOCOL: &str = "https://didcomm.org/trust_ping/1.255";
    const UNSUPPORTED_VERSION_PROTOCOL: &str = "https://didcomm.org/trust_ping/2.0";

    const KIND_PING: &str = "ping";
    const KIND_PING_RESPONSE: &str = "ping_response";

    #[test]
    fn test_protocol_trust_ping() {
        test_utils::test_protocol(PROTOCOL, TrustPingV1::V1_0(PhantomData))
    }

    #[test]
    fn test_version_resolution_trust_ping() {
        test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, TrustPingV1::V1_0(PhantomData))
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_trust_ping() {
        test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, TrustPingV1::V1_0(PhantomData))
    }

    #[test]
    fn test_msg_type_ping() {
        test_utils::test_msg_type(PROTOCOL, KIND_PING, TrustPingV1::V1_0(PhantomData))
    }

    #[test]
    fn test_msg_type_ping_response() {
        test_utils::test_msg_type(PROTOCOL, KIND_PING_RESPONSE, TrustPingV1::V1_0(PhantomData))
    }
}
