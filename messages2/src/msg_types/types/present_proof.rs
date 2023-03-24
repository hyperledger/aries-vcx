use std::marker::PhantomData;

use derive_more::{From, TryInto};
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::Protocol;
use crate::msg_types::role::Role;

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, MessageType)]
#[msg_type(protocol = "present-proof")]
pub enum PresentProofProtocol {
    V1(PresentProofProtocolV1),
}

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(PresentProofProtocol, Protocol))]
#[msg_type(major = 1)]
pub enum PresentProofProtocolV1 {
    #[msg_type(minor = 0, roles = "Role::Prover, Role::Verifier")]
    V1_0(PhantomData<fn() -> PresentProofProtocolV1_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum PresentProofProtocolV1_0 {
    ProposePresentation,
    RequestPresentation,
    Presentation,
    PresentationPreview,
    Ack,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_protocol_present_proof() {
        test_utils::test_serde(
            Protocol::from(PresentProofProtocolV1::new_v1_0()),
            json!("https://didcomm.org/present-proof/1.0"),
        )
    }

    #[test]
    fn test_version_resolution_present_proof() {
        test_utils::test_msg_type_resolution(
            "https://didcomm.org/present-proof/1.255",
            PresentProofProtocolV1::new_v1_0(),
        )
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_present_proof() {
        test_utils::test_serde(
            Protocol::from(PresentProofProtocolV1::new_v1_0()),
            json!("https://didcomm.org/present-proof/2.0"),
        )
    }

    #[test]
    fn test_msg_type_propose() {
        test_utils::test_msg_type(
            "https://didcomm.org/present-proof/1.0",
            "propose-presentation",
            PresentProofProtocolV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_request() {
        test_utils::test_msg_type(
            "https://didcomm.org/present-proof/1.0",
            "request-presentation",
            PresentProofProtocolV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_presentation() {
        test_utils::test_msg_type(
            "https://didcomm.org/present-proof/1.0",
            "presentation",
            PresentProofProtocolV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_preview() {
        test_utils::test_msg_type(
            "https://didcomm.org/present-proof/1.0",
            "presentation-preview",
            PresentProofProtocolV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_ack() {
        test_utils::test_msg_type(
            "https://didcomm.org/present-proof/1.0",
            "ack",
            PresentProofProtocolV1::new_v1_0(),
        )
    }
}
