use derive_more::{From, TryInto};
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::Transitive;

use super::Protocol;
use crate::msg_types::MsgKindType;

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, MessageType)]
#[msg_type(protocol = "signature")]
pub enum SignatureType {
    V1(SignatureTypeV1),
}

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, Transitive, MessageType)]
#[transitive(into(SignatureType, Protocol))]
#[msg_type(major = 1)]
pub enum SignatureTypeV1 {
    #[msg_type(minor = 0, roles = "")] // This is for accommodating the Connection response, it has no roles.
    V1_0(MsgKindType<SignatureTypeV1_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
pub enum SignatureTypeV1_0 {
    #[strum(serialize = "ed25519Sha512_single")]
    Ed25519Sha512Single,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_protocol_signature() {
        test_utils::test_serde(
            Protocol::from(SignatureTypeV1::new_v1_0()),
            json!("https://didcomm.org/signature/1.0"),
        )
    }

    #[test]
    fn test_version_resolution_signature() {
        test_utils::test_msg_type_resolution("https://didcomm.org/signature/1.255", SignatureTypeV1::new_v1_0())
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_signature() {
        test_utils::test_serde(
            Protocol::from(SignatureTypeV1::new_v1_0()),
            json!("https://didcomm.org/signature/2.0"),
        )
    }

    #[test]
    fn test_msg_type_sign() {
        test_utils::test_msg_type(
            "https://didcomm.org/signature/1.0",
            "ed25519Sha512_single",
            SignatureTypeV1::new_v1_0(),
        )
    }
}
