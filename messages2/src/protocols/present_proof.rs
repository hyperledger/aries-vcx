use derive_more::From;
use messages_macros::Message;
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

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "PresentProofV1_0::ProposePresentation")]
pub struct ProposePresentation;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "PresentProofV1_0::RequestPresentation")]
pub struct RequestPresentation;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "PresentProofV1_0::Presentation")]
pub struct Presentation;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "PresentProofV1_0::Ack")]
pub struct Ack;
