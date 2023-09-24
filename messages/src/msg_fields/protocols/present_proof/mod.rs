//! Module containing the `present proof` protocol messages, as defined in the [RFC](<https://github.com/hyperledger/aries-rfcs/blob/main/features/0037-present-proof/README.md>).

pub mod ack;
pub mod present;
pub mod problem_report;
pub mod propose;
pub mod request;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use self::{
    ack::{AckPresentation, AckPresentationContent},
    present::{Presentation, PresentationContent, PresentationDecorators},
    problem_report::{PresentProofProblemReport, PresentProofProblemReportContent},
    propose::{ProposePresentation, ProposePresentationContent, ProposePresentationDecorators},
    request::{RequestPresentation, RequestPresentationContent, RequestPresentationDecorators},
};
use super::{notification::ack::AckDecorators, report_problem::ProblemReportDecorators};
use crate::{
    misc::utils::{self, into_msg_with_type, transit_to_aries_msg},
    msg_fields::traits::DelayedSerde,
    msg_types::{
        protocols::present_proof::{PresentProofType, PresentProofTypeV1, PresentProofTypeV1_0},
        MsgWithType,
    },
};

#[derive(Clone, Debug, From, PartialEq)]
pub enum PresentProof {
    ProposePresentation(ProposePresentation),
    RequestPresentation(RequestPresentation),
    Presentation(Presentation),
    Ack(AckPresentation),
    ProblemReport(PresentProofProblemReport),
}

impl DelayedSerde for PresentProof {
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
            PresentProofType::V1(PresentProofTypeV1::V1_0(kind)) => kind.kind_from_str(kind_str),
        };

        match kind.map_err(D::Error::custom)? {
            PresentProofTypeV1_0::ProposePresentation => {
                ProposePresentation::deserialize(deserializer).map(From::from)
            }
            PresentProofTypeV1_0::RequestPresentation => {
                RequestPresentation::deserialize(deserializer).map(From::from)
            }
            PresentProofTypeV1_0::Presentation => {
                Presentation::deserialize(deserializer).map(From::from)
            }
            PresentProofTypeV1_0::Ack => AckPresentation::deserialize(deserializer).map(From::from),
            PresentProofTypeV1_0::ProblemReport => {
                PresentProofProblemReport::deserialize(deserializer).map(From::from)
            }
            PresentProofTypeV1_0::PresentationPreview => {
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

transit_to_aries_msg!(ProposePresentationContent: ProposePresentationDecorators, PresentProof);
transit_to_aries_msg!(RequestPresentationContent: RequestPresentationDecorators, PresentProof);
transit_to_aries_msg!(PresentationContent: PresentationDecorators, PresentProof);
transit_to_aries_msg!(AckPresentationContent: AckDecorators, PresentProof);
transit_to_aries_msg!(PresentProofProblemReportContent: ProblemReportDecorators, PresentProof);

into_msg_with_type!(
    ProposePresentation,
    PresentProofTypeV1_0,
    ProposePresentation
);
into_msg_with_type!(
    RequestPresentation,
    PresentProofTypeV1_0,
    RequestPresentation
);
into_msg_with_type!(Presentation, PresentProofTypeV1_0, Presentation);
into_msg_with_type!(AckPresentation, PresentProofTypeV1_0, Ack);
into_msg_with_type!(
    PresentProofProblemReport,
    PresentProofTypeV1_0,
    ProblemReport
);
