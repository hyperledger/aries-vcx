use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::Transitive;

use super::Protocol;
use crate::msg_types::{role::Role, MsgKindType};

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[msg_type(protocol = "revocation_notification")]
pub enum RevocationType {
    V2(RevocationTypeV2),
}

#[derive(Copy, Clone, Debug, From, PartialEq, Transitive, MessageType)]
#[transitive(into(RevocationType, Protocol))]
#[msg_type(major = 2)]
pub enum RevocationTypeV2 {
    #[msg_type(minor = 0, roles = "Role::Holder, Role::Issuer")]
    V2_0(MsgKindType<RevocationTypeV2_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum RevocationTypeV2_0 {
    Revoke,
    Ack,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_protocol_revocation_notification() {
        test_utils::test_serde(
            Protocol::from(RevocationTypeV2::new_v2_0()),
            json!("https://didcomm.org/revocation_notification/2.0"),
        )
    }

    #[test]
    fn test_version_resolution_revocation_notification() {
        test_utils::test_msg_type_resolution(
            "https://didcomm.org/revocation_notification/2.255",
            RevocationTypeV2::new_v2_0(),
        )
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_revocation_notification() {
        test_utils::test_serde(
            Protocol::from(RevocationTypeV2::new_v2_0()),
            json!("https://didcomm.org/revocation_notification/3.0"),
        )
    }

    #[test]
    fn test_msg_type_revoke() {
        test_utils::test_msg_type(
            "https://didcomm.org/revocation_notification/2.0",
            "revoke",
            RevocationTypeV2::new_v2_0(),
        )
    }

    #[test]
    fn test_msg_type_ack() {
        test_utils::test_msg_type(
            "https://didcomm.org/revocation_notification/2.0",
            "ack",
            RevocationTypeV2::new_v2_0(),
        )
    }
}
