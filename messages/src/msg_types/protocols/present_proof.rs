use derive_more::{From, TryInto};
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::Transitive;

use super::Protocol;
use crate::msg_types::{role::Role, MsgKindType};

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, MessageType)]
#[msg_type(protocol = "present-proof")]
pub enum PresentProofType {
    V1(PresentProofTypeV1),
    V2(PresentProofTypeV2),
}

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, Transitive, MessageType)]
#[transitive(into(PresentProofType, Protocol))]
#[msg_type(major = 1)]
pub enum PresentProofTypeV1 {
    #[msg_type(minor = 0, roles = "Role::Prover, Role::Verifier")]
    V1_0(MsgKindType<PresentProofTypeV1_0>),
}

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, Transitive, MessageType)]
#[transitive(into(PresentProofType, Protocol))]
#[msg_type(major = 2)]
pub enum PresentProofTypeV2 {
    #[msg_type(minor = 0, roles = "Role::Prover, Role::Verifier")]
    V2_0(MsgKindType<PresentProofTypeV2_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum PresentProofTypeV1_0 {
    ProposePresentation,
    RequestPresentation,
    Presentation,
    PresentationPreview,
    Ack,
    ProblemReport,
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum PresentProofTypeV2_0 {
    ProposePresentation,
    RequestPresentation,
    Presentation,
    PresentationPreview,
    Ack,
    ProblemReport,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_protocol_present_proof() {
        test_utils::test_serde(
            Protocol::from(PresentProofTypeV1::new_v1_0()),
            json!("https://didcomm.org/present-proof/1.0"),
        )
    }

    #[test]
    fn test_version_resolution_present_proof() {
        test_utils::test_msg_type_resolution(
            "https://didcomm.org/present-proof/1.255",
            PresentProofTypeV1::new_v1_0(),
        )
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_present_proof() {
        test_utils::test_serde(
            Protocol::from(PresentProofTypeV1::new_v1_0()),
            json!("https://didcomm.org/present-proof/2.0"),
        )
    }

    #[test]
    fn test_msg_type_propose() {
        test_utils::test_msg_type(
            "https://didcomm.org/present-proof/1.0",
            "propose-presentation",
            PresentProofTypeV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_request() {
        test_utils::test_msg_type(
            "https://didcomm.org/present-proof/1.0",
            "request-presentation",
            PresentProofTypeV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_presentation() {
        test_utils::test_msg_type(
            "https://didcomm.org/present-proof/1.0",
            "presentation",
            PresentProofTypeV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_preview() {
        test_utils::test_msg_type(
            "https://didcomm.org/present-proof/1.0",
            "presentation-preview",
            PresentProofTypeV1::new_v1_0(),
        )
    }

    #[test]
    fn test_msg_type_ack() {
        test_utils::test_msg_type(
            "https://didcomm.org/present-proof/1.0",
            "ack",
            PresentProofTypeV1::new_v1_0(),
        )
    }
}
