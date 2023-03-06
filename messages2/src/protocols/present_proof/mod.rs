mod ack;
mod present;
mod propose;
mod request;

use derive_more::From;
use serde::{Deserializer, Serializer};

use crate::{
    composite_message::{transit_to_aries_msg, Message},
    delayed_serde::DelayedSerde,
    message_type::message_family::present_proof::{PresentProof as PresentProofKind, PresentProofV1, PresentProofV1_0},
    utils,
};

use self::{
    ack::AckPresentationContent,
    present::{PresentationContent, PresentationDecorators},
    propose::{ProposePresentationContent, ProposePresentationDecorators},
    request::{RequestPresentationContent, RequestPresentationDecorators},
};

pub use self::{
    ack::AckPresentation, present::Presentation, propose::ProposePresentation, request::RequestPresentation,
};

use super::notification::AckDecorators;

#[derive(Clone, Debug, From)]
pub enum PresentProof {
    ProposePresentation(ProposePresentation),
    RequestPresentation(RequestPresentation),
    Presentation(Presentation),
    Ack(AckPresentation),
}

impl DelayedSerde for PresentProof {
    type MsgType = PresentProofKind;

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let PresentProofKind::V1(major) = msg_type;
        let PresentProofV1::V1_0(minor) = major;

        match minor {
            PresentProofV1_0::ProposePresentation => {
                ProposePresentation::delayed_deserialize(minor, deserializer).map(From::from)
            }
            PresentProofV1_0::RequestPresentation => {
                RequestPresentation::delayed_deserialize(minor, deserializer).map(From::from)
            }
            PresentProofV1_0::Presentation => Presentation::delayed_deserialize(minor, deserializer).map(From::from),
            PresentProofV1_0::Ack => AckPresentation::delayed_deserialize(minor, deserializer).map(From::from),
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

transit_to_aries_msg!(ProposePresentationContent: ProposePresentationDecorators, PresentProof);
transit_to_aries_msg!(RequestPresentationContent: RequestPresentationDecorators, PresentProof);
transit_to_aries_msg!(PresentationContent: PresentationDecorators, PresentProof);
transit_to_aries_msg!(AckPresentationContent: AckDecorators, PresentProof);
