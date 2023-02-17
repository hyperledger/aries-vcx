use derive_more::From;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{message_type::message_family::{
    present_proof::{PresentProof as PresentProofKind, PresentProofV1, PresentProofV1_0},
}, delayed_serde::DelayedSerde};

use super::traits::ConcreteMessage;

#[derive(Clone, Debug, From)]
pub enum PresentProof {
    ProposePresentation(ProposePresentation),
    RequestPresentation(RequestPresentation),
    Presentation(Presentation),
    Ack(Ack),
}

impl DelayedSerde for PresentProof {
    type MsgType = PresentProofKind;

    fn delayed_deserialize<'de, D>(seg: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let PresentProofKind::V1(major) = seg;
        let PresentProofV1::V1_0(minor) = major;

        match minor {
            PresentProofV1_0::ProposePresentation => ProposePresentation::deserialize(deserializer).map(From::from),
            PresentProofV1_0::RequestPresentation => RequestPresentation::deserialize(deserializer).map(From::from),
            PresentProofV1_0::Presentation => Presentation::deserialize(deserializer).map(From::from),
            PresentProofV1_0::Ack => Ack::deserialize(deserializer).map(From::from),
        }
    }

    fn delayed_serialize<'a, M, F, S>(&self, state: &'a mut M, closure: &mut F) -> Result<S::Ok, S::Error>
    where
        M: serde::ser::SerializeMap,
        F: FnMut(&'a mut M) -> S,
        S: serde::Serializer,
        S::Error: From<M::Error>,
    {
        match self {
            Self::ProposePresentation(v) => v.delayed_serialize(state, closure),
            Self::RequestPresentation(v) => v.delayed_serialize(state, closure),
            Self::Presentation(v) => v.delayed_serialize(state, closure),
            Self::Ack(v) => v.delayed_serialize(state, closure),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProposePresentation;

impl ConcreteMessage for ProposePresentation {
    type Kind = PresentProofV1_0;

    fn kind() -> Self::Kind {
        Self::Kind::ProposePresentation
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestPresentation;

impl ConcreteMessage for RequestPresentation {
    type Kind = PresentProofV1_0;

    fn kind() -> Self::Kind {
        Self::Kind::RequestPresentation
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Presentation;

impl ConcreteMessage for Presentation {
    type Kind = PresentProofV1_0;

    fn kind() -> Self::Kind {
        Self::Kind::Presentation
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ack;

impl ConcreteMessage for Ack {
    type Kind = PresentProofV1_0;

    fn kind() -> Self::Kind {
        Self::Kind::Ack
    }
}
