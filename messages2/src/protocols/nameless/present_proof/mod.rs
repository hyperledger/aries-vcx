//! Module containing the `present proof` protocol messages, as defined in the [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/features/0037-present-proof/README.md).

pub mod ack;
pub mod present;
pub mod propose;
pub mod request;

use std::str::FromStr;

use derive_more::From;
use serde::{de::Error, Deserializer, Serializer};

use self::{
    ack::{AckPresentationContent, AckPresentation},
    present::{Presentation, PresentationContent, PresentationDecorators},
    propose::{ProposePresentation, ProposePresentationContent, ProposePresentationDecorators},
    request::{RequestPresentation, RequestPresentationContent, RequestPresentationDecorators},
};
use super::notification::AckDecorators;
use crate::{
    misc::utils::{self, transit_to_aries_msg},
    msg_types::types::present_proof::{PresentProof as PresentProofKind, PresentProofV1, PresentProofV1_0},
    protocols::traits::DelayedSerde,
};

#[derive(Clone, Debug, From, PartialEq)]
pub enum PresentProof {
    ProposePresentation(ProposePresentation),
    RequestPresentation(RequestPresentation),
    Presentation(Presentation),
    Ack(AckPresentation),
}

impl DelayedSerde for PresentProof {
    type MsgType<'a> = (PresentProofKind, &'a str);

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (major, kind) = msg_type;
        let PresentProofKind::V1(major) = major;
        let PresentProofV1::V1_0(_minor) = major;
        let kind = PresentProofV1_0::from_str(kind).map_err(D::Error::custom)?;

        match kind {
            PresentProofV1_0::ProposePresentation => {
                ProposePresentation::delayed_deserialize(kind, deserializer).map(From::from)
            }
            PresentProofV1_0::RequestPresentation => {
                RequestPresentation::delayed_deserialize(kind, deserializer).map(From::from)
            }
            PresentProofV1_0::Presentation => Presentation::delayed_deserialize(kind, deserializer).map(From::from),
            PresentProofV1_0::Ack => AckPresentation::delayed_deserialize(kind, deserializer).map(From::from),
            PresentProofV1_0::PresentationPreview => Err(utils::not_standalone_msg::<D>(kind.as_ref())),
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
