use derive_more::{From, TryInto};
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::Transitive;

use super::Protocol;
use crate::msg_types::{role::Role, MsgKindType};

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, MessageType)]
#[msg_type(protocol = "connections")]
pub enum ConnectionType {
    V1(ConnectionTypeV1),
}

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, Transitive, MessageType)]
#[transitive(into(ConnectionType, Protocol))]
#[msg_type(major = 1)]
pub enum ConnectionTypeV1 {
    #[msg_type(minor = 0, roles = "Role::Inviter, Role::Invitee")]
    V1_0(MsgKindType<ConnectionTypeV1_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum ConnectionTypeV1_0 {
    Invitation,
    Request,
    Response,
    ProblemReport,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_protocol_connections() {
        test_utils::test_serde(
            Protocol::from(ConnectionTypeV1::new_v1_0()),
            json!("https://didcomm.org/connections/1.0"),
        )
    }

    #[test]
    fn test_version_resolution_connections() {
        test_utils::test_msg_type_resolution(
            "https://didcomm.org/connections/1.255",
            ConnectionTypeV1::new_v1_0(),
        )
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_connections() {
        test_utils::test_serde(
            Protocol::from(ConnectionTypeV1::new_v1_0()),
            json!("https://didcomm.org/connections/2.0"),
        )
    }

    #[test]
    fn test_msg_type_invitation() {
        test_utils::test_msg_type(
            "https://didcomm.org/connections/1.0",
            "invitation",
            ConnectionTypeV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_request() {
        test_utils::test_msg_type(
            "https://didcomm.org/connections/1.0",
            "request",
            ConnectionTypeV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_response() {
        test_utils::test_msg_type(
            "https://didcomm.org/connections/1.0",
            "response",
            ConnectionTypeV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_problem() {
        test_utils::test_msg_type(
            "https://didcomm.org/connections/1.0",
            "problem_report",
            ConnectionTypeV1::new_v1_0(),
        )
    }
}
