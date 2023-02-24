mod ack;
mod present;
mod propose;
mod request;

use derive_more::From;
use serde::{Deserialize, Deserializer, Serializer};

use crate::{
    delayed_serde::DelayedSerde,
    message_type::message_family::present_proof::{PresentProof as PresentProofKind, PresentProofV1, PresentProofV1_0},
    utils,
};

use self::{ack::AckPresentation, present::Presentation, propose::ProposePresentation, request::RequestPresentation};

#[derive(Clone, Debug, From)]
pub enum PresentProof {
    ProposePresentation(ProposePresentation),
    RequestPresentation(RequestPresentation),
    Presentation(Presentation),
    Ack(AckPresentation),
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
            PresentProofV1_0::Ack => AckPresentation::deserialize(deserializer).map(From::from),
            PresentProofV1_0::PresentationPreview => Err(utils::not_standalone_msg::<D>(minor.as_ref())),
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::ProposePresentation(v) => v.delayed_serialize(serializer),
            Self::RequestPresentation(v) => v.delayed_serialize(serializer),
            Self::Presentation(v) => v.delayed_serialize(serializer),
            Self::Ack(v) => v.delayed_serialize(serializer),
        }
    }
}
