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
#[semver(protocol = "trust_ping")]
pub enum TrustPing {
    V1(TrustPingV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(TrustPing, Protocol)))]
#[semver(major = 1, parent = "TrustPing", actors(Actor::Sender, Actor::Receiver))]
pub enum TrustPingV1 {
    V1_0(TrustPingV1_0),
}

#[derive(Copy, Clone, Debug, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(TrustPingV1, TrustPing, Protocol)))]
#[semver(minor = 0, parent = "TrustPingV1")]
pub struct TrustPingV1_0;

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, MessageType)]
#[strum(serialize_all = "kebab-case")]
#[semver(parent = "TrustPingV1_0")]
pub enum TrustPingV1_0Kind {
    Ping,
    #[strum(serialize = "ping_response")]
    PingResponse,
}

#[cfg(test)]
mod tests {
    use super::TrustPingV1_0;
    use crate::misc::test_utils;

    const PROTOCOL: &str = "https://didcomm.org/trust_ping/1.0";
    const VERSION_RESOLUTION_PROTOCOL: &str = "https://didcomm.org/trust_ping/1.255";
    const UNSUPPORTED_VERSION_PROTOCOL: &str = "https://didcomm.org/trust_ping/2.0";

    const KIND_PING: &str = "ping";
    const KIND_PING_RESPONSE: &str = "ping_response";

    #[test]
    fn test_protocol_trust_ping() {
        test_utils::test_protocol(PROTOCOL, TrustPingV1_0)
    }

    #[test]
    fn test_version_resolution_trust_ping() {
        test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, TrustPingV1_0)
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_trust_ping() {
        test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, TrustPingV1_0)
    }

    #[test]
    fn test_msg_type_ping() {
        test_utils::test_msg_type(PROTOCOL, KIND_PING, TrustPingV1_0)
    }

    #[test]
    fn test_msg_type_ping_response() {
        test_utils::test_msg_type(PROTOCOL, KIND_PING_RESPONSE, TrustPingV1_0)
    }
}
