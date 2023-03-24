//! Module containing the `present proof` protocol messages, as defined in the [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/features/0037-present-proof/README.md).

pub mod ack;
pub mod present;
pub mod propose;
pub mod request;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use self::{
    ack::{AckPresentation, AckPresentationContent},
    present::{Presentation, PresentationContent, PresentationDecorators},
    propose::{ProposePresentation, ProposePresentationContent, ProposePresentationDecorators},
    request::{RequestPresentation, RequestPresentationContent, RequestPresentationDecorators},
};
use super::notification::AckDecorators;
use crate::{
    misc::utils::{self, into_msg_with_type, transit_to_aries_msg},
    msg_types::{
        types::present_proof::{PresentProofProtocol, PresentProofProtocolV1, PresentProofProtocolV1_0},
        MsgWithType,
    },
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
    type MsgType<'a> = (PresentProofProtocol, &'a str);

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (major, kind_str) = msg_type;

        let kind = match major {
            PresentProofProtocol::V1(PresentProofProtocolV1::V1_0(kind)) => kind.kind_from_str(kind_str),
        };

        match kind.map_err(D::Error::custom)? {
            PresentProofProtocolV1_0::ProposePresentation => {
                ProposePresentation::deserialize(deserializer).map(From::from)
            }
            PresentProofProtocolV1_0::RequestPresentation => {
                RequestPresentation::deserialize(deserializer).map(From::from)
            }
            PresentProofProtocolV1_0::Presentation => Presentation::deserialize(deserializer).map(From::from),
            PresentProofProtocolV1_0::Ack => AckPresentation::deserialize(deserializer).map(From::from),
            PresentProofProtocolV1_0::PresentationPreview => Err(utils::not_standalone_msg::<D>(kind_str)),
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::ProposePresentation(v) => MsgWithType::from(v).serialize(serializer),
            Self::RequestPresentation(v) => MsgWithType::from(v).serialize(serializer),
            Self::Presentation(v) => MsgWithType::from(v).serialize(serializer),
            Self::Ack(v) => MsgWithType::from(v).serialize(serializer),
        }
    }
}

transit_to_aries_msg!(ProposePresentationContent: ProposePresentationDecorators, PresentProof);
transit_to_aries_msg!(RequestPresentationContent: RequestPresentationDecorators, PresentProof);
transit_to_aries_msg!(PresentationContent: PresentationDecorators, PresentProof);
transit_to_aries_msg!(AckPresentationContent: AckDecorators, PresentProof);

into_msg_with_type!(ProposePresentation, PresentProofProtocolV1_0, ProposePresentation);
into_msg_with_type!(RequestPresentation, PresentProofProtocolV1_0, RequestPresentation);
into_msg_with_type!(Presentation, PresentProofProtocolV1_0, Presentation);
into_msg_with_type!(AckPresentation, PresentProofProtocolV1_0, Ack);
