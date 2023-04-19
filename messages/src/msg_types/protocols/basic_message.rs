use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::Protocol;
use crate::msg_types::{role::Role, MsgKindType};

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[msg_type(protocol = "basicmessage")]
pub enum BasicMessageType {
    V1(BasicMessageTypeV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(BasicMessageType, Protocol))]
#[msg_type(major = 1)]
pub enum BasicMessageTypeV1 {
    #[msg_type(minor = 0, roles = "Role::Receiver, Role::Sender")]
    V1_0(MsgKindType<BasicMessageTypeV1_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum BasicMessageTypeV1_0 {
    Message,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_protocol_basic_message() {
        test_utils::test_serde(
            Protocol::from(BasicMessageTypeV1::new_v1_0()),
            json!("https://didcomm.org/basicmessage/1.0"),
        )
    }

    #[test]
    fn test_version_resolution_basic_message() {
        test_utils::test_msg_type_resolution("https://didcomm.org/basicmessage/1.255", BasicMessageTypeV1::new_v1_0())
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_basic_message() {
        test_utils::test_serde(
            Protocol::from(BasicMessageTypeV1::new_v1_0()),
            json!("https://didcomm.org/basicmessage/2.0"),
        )
    }

    #[test]
    fn test_msg_type_message() {
        test_utils::test_msg_type(
            "https://didcomm.org/basicmessage/1.0",
            "message",
            BasicMessageTypeV1::new_v1_0(),
        )
    }
}
