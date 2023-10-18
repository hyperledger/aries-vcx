//! Module containing the `present proof` protocol messages, as defined in the [RFC](<https://github.com/hyperledger/aries-rfcs/blob/main/features/0037-present-proof/README.md>).

pub mod ack;
pub mod present;
pub mod problem_report;
pub mod propose;
pub mod request;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use self::{
    ack::{AckPresentationV2, AckPresentationV2Content},
    present::{PresentationV2, PresentationV2Content, PresentationV2Decorators},
    problem_report::{PresentProofV2ProblemReport, PresentProofV2ProblemReportContent},
    propose::{
        ProposePresentationV2, ProposePresentationV2Content, ProposePresentationV2Decorators,
    },
    request::{
        RequestPresentationV2, RequestPresentationV2Content, RequestPresentationV2Decorators,
    },
};
use super::PresentProof;
use crate::{
    misc::utils::{self, into_msg_with_type, transit_to_aries_msg},
    msg_fields::{
        protocols::{notification::ack::AckDecorators, report_problem::ProblemReportDecorators},
        traits::DelayedSerde,
    },
    msg_types::{
        present_proof::{PresentProofTypeV2, PresentProofTypeV2_0},
        protocols::present_proof::{PresentProofType, PresentProofTypeV1_0},
        MsgWithType,
    },
};

#[derive(Clone, Debug, From, PartialEq)]
pub enum PresentProofV2 {
    ProposePresentation(ProposePresentationV2),
    RequestPresentation(RequestPresentationV2),
    Presentation(PresentationV2),
    Ack(AckPresentationV2),
    ProblemReport(PresentProofV2ProblemReport),
}

impl DelayedSerde for PresentProofV2 {
    type MsgType<'a> = (PresentProofType, &'a str);

    fn delayed_deserialize<'de, D>(
        msg_type: Self::MsgType<'de>,
        deserializer: D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (protocol, kind_str) = msg_type;

        let kind = match protocol {
            PresentProofType::V2(PresentProofTypeV2::V2_0(kind)) => kind.kind_from_str(kind_str),
            PresentProofType::V1(_) => {
                return Err(D::Error::custom(
                    "Cannot deserialize present-proof-v1 message type into present-proof-v2",
                ))
            }
        };

        match kind.map_err(D::Error::custom)? {
            PresentProofTypeV2_0::ProposePresentation => {
                ProposePresentationV2::deserialize(deserializer).map(From::from)
            }
            PresentProofTypeV2_0::RequestPresentation => {
                RequestPresentationV2::deserialize(deserializer).map(From::from)
            }
            PresentProofTypeV2_0::Presentation => {
                PresentationV2::deserialize(deserializer).map(From::from)
            }
            PresentProofTypeV2_0::Ack => {
                AckPresentationV2::deserialize(deserializer).map(From::from)
            }
            PresentProofTypeV2_0::ProblemReport => {
                PresentProofV2ProblemReport::deserialize(deserializer).map(From::from)
            }
            PresentProofTypeV2_0::PresentationPreview => {
                Err(utils::not_standalone_msg::<D>(kind_str))
            }
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
            Self::ProblemReport(v) => MsgWithType::from(v).serialize(serializer),
        }
    }
}

transit_to_aries_msg!(
    ProposePresentationV2Content: ProposePresentationV2Decorators,
    PresentProofV2, PresentProof
);
transit_to_aries_msg!(
    RequestPresentationV2Content: RequestPresentationV2Decorators,
    PresentProofV2, PresentProof
);
transit_to_aries_msg!(PresentationV2Content: PresentationV2Decorators, PresentProofV2, PresentProof);
transit_to_aries_msg!(AckPresentationV2Content: AckDecorators, PresentProofV2, PresentProof);
transit_to_aries_msg!(
    PresentProofV2ProblemReportContent: ProblemReportDecorators,
    PresentProofV2, PresentProof
);

into_msg_with_type!(
    ProposePresentationV2,
    PresentProofTypeV1_0,
    ProposePresentation
);
into_msg_with_type!(
    RequestPresentationV2,
    PresentProofTypeV1_0,
    RequestPresentation
);
into_msg_with_type!(PresentationV2, PresentProofTypeV1_0, Presentation);
into_msg_with_type!(AckPresentationV2, PresentProofTypeV1_0, Ack);
into_msg_with_type!(
    PresentProofV2ProblemReport,
    PresentProofTypeV1_0,
    ProblemReport
);
