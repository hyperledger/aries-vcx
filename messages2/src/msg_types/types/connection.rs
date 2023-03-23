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
    #[msg_type(minor = 0, roles = "Role::Inviter, Role::Invitee")]
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
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_protocol_connections() {
        test_utils::test_serde(
            Protocol::from(ConnectionV1::new_v1_0()),
            json!("https://didcomm.org/connections/1.0"),
        )
    }

    #[test]
    fn test_version_resolution_connections() {
        test_utils::test_msg_type_resolution("https://didcomm.org/connections/1.255", ConnectionV1::new_v1_0())
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_connections() {
        test_utils::test_serde(
            Protocol::from(ConnectionV1::new_v1_0()),
            json!("https://didcomm.org/connections/2.0"),
        )
    }

    #[test]
    fn test_msg_type_invitation() {
        test_utils::test_msg_type(
            "https://didcomm.org/connections/1.0",
            "invitation",
            ConnectionV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_request() {
        test_utils::test_msg_type(
            "https://didcomm.org/connections/1.0",
            "request",
            ConnectionV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_response() {
        test_utils::test_msg_type(
            "https://didcomm.org/connections/1.0",
            "response",
            ConnectionV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_problem() {
        test_utils::test_msg_type(
            "https://didcomm.org/connections/1.0",
            "problem_report",
            ConnectionV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_sign() {
        test_utils::test_msg_type(
            "https://didcomm.org/connections/1.0",
            "ed25519Sha512_single",
            ConnectionV1::new_v1_0(),
        )
    }
}
