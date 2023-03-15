use derive_more::{From, TryInto};
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::{
    traits::{MajorVersion, MinorVersion, ProtocolName},
    Protocol,
};
use crate::msg_types::{actor::Actor, registry::get_supported_version};

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, MessageType)]
#[semver(protocol = "connections")]
pub enum Connection {
    V1(ConnectionV1),
}

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(Connection, Protocol)))]
#[semver(major = 1, parent = "Connection", actors(Actor::Inviter, Actor::Invitee))]
pub enum ConnectionV1 {
    V1_0(ConnectionV1_0),
}

#[derive(Copy, Clone, Debug, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(ConnectionV1, Connection, Protocol)))]
#[semver(minor = 0, parent = "ConnectionV1")]
pub struct ConnectionV1_0;

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, MessageType)]
#[strum(serialize_all = "snake_case")]
#[semver(parent = "ConnectionV1_0")]
pub enum ConnectionV1_0Kind {
    Invitation,
    Request,
    Response,
    ProblemReport,
    #[strum(serialize = "ed25519Sha512_single")]
    Ed25519Sha512Single,
}

#[cfg(test)]
mod tests {
    use crate::misc::test_utils;

    use super::ConnectionV1_0;

    const PROTOCOL: &str = "https://didcomm.org/connections/1.0";
    const VERSION_RESOLUTION_PROTOCOL: &str = "https://didcomm.org/connections/1.255";
    const UNSUPPORTED_VERSION_PROTOCOL: &str = "https://didcomm.org/connections/2.0";

    const KIND_INVITATION: &str = "invitation";
    const KIND_REQUEST: &str = "request";
    const KIND_RESPONSE: &str = "response";
    const KIND_PROBLEM: &str = "problem_report";
    const KIND_SIGN: &str = "ed25519Sha512_single";

    #[test]
    fn test_protocol_connections() {
        test_utils::test_protocol(PROTOCOL, ConnectionV1_0)
    }

    #[test]
    fn test_version_resolution_connections() {
        test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, ConnectionV1_0)
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_connections() {
        test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, ConnectionV1_0)
    }

    #[test]
    fn test_msg_type_invitation() {
        test_utils::test_msg_type(PROTOCOL, KIND_INVITATION, ConnectionV1_0)
    }

    #[test]
    fn test_msg_type_request() {
        test_utils::test_msg_type(PROTOCOL, KIND_REQUEST, ConnectionV1_0)
    }

    #[test]
    fn test_msg_type_response() {
        test_utils::test_msg_type(PROTOCOL, KIND_RESPONSE, ConnectionV1_0)
    }

    #[test]
    fn test_msg_type_problem() {
        test_utils::test_msg_type(PROTOCOL, KIND_PROBLEM, ConnectionV1_0)
    }

    #[test]
    fn test_msg_type_sign() {
        test_utils::test_msg_type(PROTOCOL, KIND_SIGN, ConnectionV1_0)
    }
}
