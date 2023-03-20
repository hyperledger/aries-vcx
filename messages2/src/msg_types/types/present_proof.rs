use std::marker::PhantomData;

use derive_more::{From, TryInto};
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use super::Protocol;
use crate::msg_types::actor::Actor;

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, MessageType)]
#[msg_type(protocol = "present-proof")]
pub enum PresentProof {
    V1(PresentProofV1),
}

#[derive(Copy, Clone, Debug, From, TryInto, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(PresentProof, Protocol)))]
#[msg_type(major = 1)]
pub enum PresentProofV1 {
    #[msg_type(minor = 0, actors = "Actor::Prover, Actor::Verifier")]
    V1_0(PhantomData<PresentProofV1_0Kind>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum PresentProofV1_0Kind {
    ProposePresentation,
    RequestPresentation,
    Presentation,
    PresentationPreview,
    Ack,
}

// #[cfg(test)]
// mod tests {
//     use super::PresentProofV1_0;
//     use crate::misc::test_utils;

//     const PROTOCOL: &str = "https://didcomm.org/present-proof/1.0";
//     const VERSION_RESOLUTION_PROTOCOL: &str = "https://didcomm.org/present-proof/1.255";
//     const UNSUPPORTED_VERSION_PROTOCOL: &str = "https://didcomm.org/present-proof/2.0";

//     const KIND_PROPOSE: &str = "propose-presentation";
//     const KIND_REQUEST: &str = "request-presentation";
//     const KIND_PRESENTATION: &str = "presentation";
//     const KIND_PREVIEW: &str = "presentation-preview";
//     const KIND_ACK: &str = "ack";

//     #[test]
//     fn test_protocol_present_proof() {
//         test_utils::test_protocol(PROTOCOL, PresentProofV1_0)
//     }

//     #[test]
//     fn test_version_resolution_present_proof() {
//         test_utils::test_protocol(VERSION_RESOLUTION_PROTOCOL, PresentProofV1_0)
//     }

//     #[test]
//     #[should_panic]
//     fn test_unsupported_version_present_proof() {
//         test_utils::test_protocol(UNSUPPORTED_VERSION_PROTOCOL, PresentProofV1_0)
//     }

//     #[test]
//     fn test_msg_type_propose() {
//         test_utils::test_msg_type(PROTOCOL, KIND_PROPOSE, PresentProofV1_0)
//     }

//     #[test]
//     fn test_msg_type_request() {
//         test_utils::test_msg_type(PROTOCOL, KIND_REQUEST, PresentProofV1_0)
//     }

//     #[test]
//     fn test_msg_type_presentation() {
//         test_utils::test_msg_type(PROTOCOL, KIND_PRESENTATION, PresentProofV1_0)
//     }

//     #[test]
//     fn test_msg_type_preview() {
//         test_utils::test_msg_type(PROTOCOL, KIND_PREVIEW, PresentProofV1_0)
//     }

//     #[test]
//     fn test_msg_type_ack() {
//         test_utils::test_msg_type(PROTOCOL, KIND_ACK, PresentProofV1_0)
//     }
// }
