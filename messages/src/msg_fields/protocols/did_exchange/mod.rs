// TODO: Why are not msg fields and types grouped by protocol???
pub mod complete;
// TODO: Duplicates connection problem report, deduplicate
pub mod problem_report;
pub mod request;
pub mod response;

use derive_more::From;
use serde::{de::Error, Deserialize, Serialize};
use shared::misc::serde_ignored::SerdeIgnored as NoContent;

use self::{
    complete::{Complete, CompleteDecorators},
    problem_report::{ProblemReport, ProblemReportContent, ProblemReportDecorators},
    request::{Request, RequestContent, RequestDecorators},
    response::{Response, ResponseContent, ResponseDecorators},
};
use crate::{
    misc::utils::{into_msg_with_type, transit_to_aries_msg},
    msg_fields::traits::DelayedSerde,
    msg_types::{
        protocols::did_exchange::{
            DidExchangeType as DidExchangeKind, DidExchangeTypeV1, DidExchangeTypeV1_0,
        },
        MsgWithType,
    },
};

#[derive(Clone, Debug, From, PartialEq)]
pub enum DidExchange {
    Request(Request),
    Response(Response),
    ProblemReport(ProblemReport),
    Complete(Complete),
}

impl DelayedSerde for DidExchange {
    type MsgType<'a> = (DidExchangeKind, &'a str);

    fn delayed_deserialize<'de, D>(
        msg_type: Self::MsgType<'de>,
        deserializer: D,
    ) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (protocol, kind_str) = msg_type;

        let kind = match protocol {
            DidExchangeKind::V1(DidExchangeTypeV1::V1_0(kind)) => kind.kind_from_str(kind_str),
        };

        match kind.map_err(D::Error::custom)? {
            DidExchangeTypeV1_0::Request => Request::deserialize(deserializer).map(From::from),
            DidExchangeTypeV1_0::Response => Response::deserialize(deserializer).map(From::from),
            DidExchangeTypeV1_0::ProblemReport => {
                ProblemReport::deserialize(deserializer).map(From::from)
            }
            DidExchangeTypeV1_0::Complete => Complete::deserialize(deserializer).map(From::from),
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Request(v) => MsgWithType::from(v).serialize(serializer),
            Self::Response(v) => MsgWithType::from(v).serialize(serializer),
            Self::ProblemReport(v) => MsgWithType::from(v).serialize(serializer),
            Self::Complete(v) => MsgWithType::from(v).serialize(serializer),
        }
    }
}

// TODO: Seems to be required only for tests?
transit_to_aries_msg!(RequestContent: RequestDecorators, DidExchange);
transit_to_aries_msg!(ResponseContent: ResponseDecorators, DidExchange);
transit_to_aries_msg!(ProblemReportContent: ProblemReportDecorators, DidExchange);
transit_to_aries_msg!(NoContent: CompleteDecorators, DidExchange);

into_msg_with_type!(Request, DidExchangeTypeV1_0, Request);
into_msg_with_type!(Response, DidExchangeTypeV1_0, Response);
into_msg_with_type!(ProblemReport, DidExchangeTypeV1_0, ProblemReport);
into_msg_with_type!(Complete, DidExchangeTypeV1_0, Complete);
