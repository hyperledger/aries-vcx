use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::Protocol;
use crate::msg_types::{role::Role, MsgKindType};

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[msg_type(protocol = "out-of-band")]
pub enum OutOfBandType {
    V1(OutOfBandTypeV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(OutOfBandType, Protocol))]
#[msg_type(major = 1)]
pub enum OutOfBandTypeV1 {
    #[msg_type(minor = 1, roles = "Role::Receiver, Role::Sender")]
    V1_1(MsgKindType<OutOfBandTypeV1_1>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum OutOfBandTypeV1_1 {
    Invitation,
    HandshakeReuse,
    HandshakeReuseAccepted,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_protocol_out_of_band() {
        test_utils::test_serde(
            Protocol::from(OutOfBandTypeV1::new_v1_1()),
            json!("https://didcomm.org/out-of-band/1.1"),
        )
    }

    #[test]
    fn test_version_resolution_out_of_band() {
        test_utils::test_msg_type_resolution("https://didcomm.org/out-of-band/1.255", OutOfBandTypeV1::new_v1_1())
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_out_of_band() {
        test_utils::test_serde(
            Protocol::from(OutOfBandTypeV1::new_v1_1()),
            json!("https://didcomm.org/out-of-band/2.0"),
        )
    }

    #[test]
    fn test_msg_type_invitation() {
        test_utils::test_msg_type(
            "https://didcomm.org/out-of-band/1.1",
            "invitation",
            OutOfBandTypeV1::new_v1_1(),
        )
    }

    #[test]
    fn test_msg_type_reuse() {
        test_utils::test_msg_type(
            "https://didcomm.org/out-of-band/1.1",
            "handshake-reuse",
            OutOfBandTypeV1::new_v1_1(),
        )
    }

    #[test]
    fn test_msg_type_reuse_acc() {
        test_utils::test_msg_type(
            "https://didcomm.org/out-of-band/1.1",
            "handshake-reuse-accepted",
            OutOfBandTypeV1::new_v1_1(),
        )
    }
}
