use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use crate::{
    error::{MsgTypeError, MsgTypeResult},
    message_type::MessageType,
};

use super::{
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    MessageFamily,
};

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(MessageFamily, MessageType))]
#[semver(family = "present-proof")]
pub enum PresentProof {
    V1(PresentProofV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(PresentProof, MessageFamily, MessageType)))]
#[semver(major = 1)]
pub enum PresentProofV1 {
    V1_0(PresentProofV1_0),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(PresentProofV1, PresentProof, MessageFamily, MessageType)))]
#[strum(serialize_all = "kebab-case")]
#[semver(minor = 0)]
pub enum PresentProofV1_0 {
    ProposePresentation,
    RequestPresentation,
    Presentation,
    Ack,
}
