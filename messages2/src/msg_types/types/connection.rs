use std::marker::PhantomData;

use derive_more::{From, TryInto};
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::Protocol;
use crate::msg_types::role::Role;

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, MessageType)]
#[msg_type(protocol = "connections")]
pub enum Connection {
    V1(ConnectionV1),
}

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(Connection, Protocol))]
#[msg_type(major = 1)]
pub enum ConnectionV1 {
    #[msg_type(minor = 0, actors = "Role::Inviter, Role::Invitee")]
    V1_0(PhantomData<fn() -> ConnectionV1_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum ConnectionV1_0 {
    Invitation,
    Request,
    Response,
    ProblemReport,
    #[strum(serialize = "ed25519Sha512_single")]
    Ed25519Sha512Single,
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::ConnectionV1;
    use crate::misc::test_utils;

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
        test_utils::test_protocol(PROTOCOL, ConnectionV1::V1_0(PhantomData))
    }

    #[test]
    fn test_version_resolution_connections() {
        test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, ConnectionV1::V1_0(PhantomData))
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_connections() {
        test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, ConnectionV1::V1_0(PhantomData))
    }

    #[test]
    fn test_msg_type_invitation() {
        test_utils::test_msg_type(PROTOCOL, KIND_INVITATION, ConnectionV1::V1_0(PhantomData))
    }

    #[test]
    fn test_msg_type_request() {
        test_utils::test_msg_type(PROTOCOL, KIND_REQUEST, ConnectionV1::V1_0(PhantomData))
    }

    #[test]
    fn test_msg_type_response() {
        test_utils::test_msg_type(PROTOCOL, KIND_RESPONSE, ConnectionV1::V1_0(PhantomData))
    }

    #[test]
    fn test_msg_type_problem() {
        test_utils::test_msg_type(PROTOCOL, KIND_PROBLEM, ConnectionV1::V1_0(PhantomData))
    }

    #[test]
    fn test_msg_type_sign() {
        test_utils::test_msg_type(PROTOCOL, KIND_SIGN, ConnectionV1::V1_0(PhantomData))
    }
}
