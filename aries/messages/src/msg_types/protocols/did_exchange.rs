use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::Transitive;

use super::Protocol;
use crate::msg_types::{role::Role, MsgKindType};

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[msg_type(protocol = "didexchange")]
pub enum DidExchangeType {
    V1(DidExchangeTypeV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, Transitive, MessageType)]
#[transitive(into(DidExchangeType, Protocol))]
#[msg_type(major = 1)]
pub enum DidExchangeTypeV1 {
    #[msg_type(minor = 0, roles = "Role::Requester, Role::Responder")]
    V1_0(MsgKindType<DidExchangeTypeV1_0>),
    #[msg_type(minor = 1, roles = "Role::Requester, Role::Responder")]
    V1_1(MsgKindType<DidExchangeTypeV1_1>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum DidExchangeTypeV1_0 {
    Request,
    Response,
    ProblemReport,
    Complete,
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum DidExchangeTypeV1_1 {
    Request,
    Response,
    ProblemReport,
    Complete,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_protocol_didexchange() {
        test_utils::test_serde(
            Protocol::from(DidExchangeTypeV1::new_v1_0()),
            json!("https://didcomm.org/didexchange/1.0"),
        )
    }

    #[test]
    fn test_version_resolution_didexchange() {
        test_utils::test_msg_type_resolution(
            "https://didcomm.org/didexchange/1.255",
            DidExchangeTypeV1::new_v1_0(),
        )
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_didexchange() {
        test_utils::test_serde(
            Protocol::from(DidExchangeTypeV1::new_v1_0()),
            json!("https://didcomm.org/didexchange/2.0"),
        )
    }

    #[test]
    fn test_msg_type_request() {
        test_utils::test_msg_type(
            "https://didcomm.org/didexchange/1.0",
            "request",
            DidExchangeTypeV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_response() {
        test_utils::test_msg_type(
            "https://didcomm.org/didexchange/1.0",
            "response",
            DidExchangeTypeV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_complete() {
        test_utils::test_msg_type(
            "https://didcomm.org/didexchange/1.0",
            "complete",
            DidExchangeTypeV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_problem() {
        test_utils::test_msg_type(
            "https://didcomm.org/didexchange/1.0",
            "problem_report",
            DidExchangeTypeV1::new_v1_0(),
        )
    }
}
