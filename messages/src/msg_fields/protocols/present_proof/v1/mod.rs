//! Module containing the `present proof` protocol messages, as defined in the [RFC](<https://github.com/hyperledger/aries-rfcs/blob/main/features/0037-present-proof/README.md>).

pub mod ack;
pub mod present;
pub mod problem_report;
pub mod propose;
pub mod request;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use self::{
    ack::{AckPresentationV1, AckPresentationV1Content},
    present::{PresentationV1, PresentationV1Content, PresentationV1Decorators},
    problem_report::{PresentProofV1ProblemReport, PresentProofV1ProblemReportContent},
    propose::{
        ProposePresentationV1, ProposePresentationV1Content, ProposePresentationV1Decorators,
    },
    request::{
        RequestPresentationV1, RequestPresentationV1Content, RequestPresentationV1Decorators,
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
        protocols::present_proof::{PresentProofType, PresentProofTypeV1, PresentProofTypeV1_0},
        MsgWithType,
    },
};

#[derive(Clone, Debug, From, PartialEq)]
pub enum PresentProofV1 {
    ProposePresentation(ProposePresentationV1),
    RequestPresentation(RequestPresentationV1),
    Presentation(PresentationV1),
    Ack(AckPresentationV1),
    ProblemReport(PresentProofV1ProblemReport),
}

impl DelayedSerde for PresentProofV1 {
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
                ProposePresentationV1::deserialize(deserializer).map(From::from)
            }
            PresentProofTypeV1_0::RequestPresentation => {
                RequestPresentationV1::deserialize(deserializer).map(From::from)
            }
            PresentProofTypeV1_0::Presentation => {
                PresentationV1::deserialize(deserializer).map(From::from)
            }
            PresentProofTypeV1_0::Ack => {
                AckPresentationV1::deserialize(deserializer).map(From::from)
            }
            PresentProofTypeV1_0::ProblemReport => {
                PresentProofV1ProblemReport::deserialize(deserializer).map(From::from)
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

transit_to_aries_msg!(
    ProposePresentationV1Content: ProposePresentationV1Decorators,
    PresentProofV1, PresentProof
);
transit_to_aries_msg!(
    RequestPresentationV1Content: RequestPresentationV1Decorators,
    PresentProofV1, PresentProof
);
transit_to_aries_msg!(PresentationV1Content: PresentationV1Decorators, PresentProofV1, PresentProof);
transit_to_aries_msg!(AckPresentationV1Content: AckDecorators, PresentProofV1, PresentProof);
transit_to_aries_msg!(
    PresentProofV1ProblemReportContent: ProblemReportDecorators,
    PresentProofV1, PresentProof
);

into_msg_with_type!(
    ProposePresentationV1,
    PresentProofTypeV1_0,
    ProposePresentation
);
into_msg_with_type!(
    RequestPresentationV1,
    PresentProofTypeV1_0,
    RequestPresentation
);
into_msg_with_type!(PresentationV1, PresentProofTypeV1_0, Presentation);
into_msg_with_type!(AckPresentationV1, PresentProofTypeV1_0, Ack);
into_msg_with_type!(
    PresentProofV1ProblemReport,
    PresentProofTypeV1_0,
    ProblemReport
);
