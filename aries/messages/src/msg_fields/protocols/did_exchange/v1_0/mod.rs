pub mod complete;
pub mod problem_report;
pub mod request;
pub mod response;

use derive_more::From;
use serde::{de::Error, Deserialize, Serialize};

use self::{
    complete::Complete,
    problem_report::ProblemReport,
    request::Request,
    response::{Response, ResponseContent},
};
use super::{v1_x::response::ResponseDecorators, DidExchange};
use crate::{
    misc::utils::{into_msg_with_type, transit_to_aries_msg},
    msg_fields::traits::DelayedSerde,
    msg_types::{
        protocols::did_exchange::{DidExchangeTypeV1, DidExchangeTypeV1_0},
        MsgKindType, MsgWithType,
    },
};

#[derive(Clone, Debug, From, PartialEq)]
pub enum DidExchangeV1_0 {
    Request(Request),
    Response(Response),
    ProblemReport(ProblemReport),
    Complete(Complete),
}

impl DelayedSerde for DidExchangeV1_0 {
    type MsgType<'a> = (MsgKindType<DidExchangeTypeV1_0>, &'a str);

    fn delayed_deserialize<'de, D>(
        msg_type: Self::MsgType<'de>,
        deserializer: D,
    ) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (protocol, kind_str) = msg_type;
        let kind = protocol.kind_from_str(kind_str);

        match kind.map_err(D::Error::custom)? {
            DidExchangeTypeV1_0::Request => Request::deserialize(deserializer).map(|mut c| {
                c.content.version = DidExchangeTypeV1::new_v1_0();
                c.into()
            }),
            DidExchangeTypeV1_0::Response => Response::deserialize(deserializer).map(From::from),
            DidExchangeTypeV1_0::ProblemReport => {
                ProblemReport::deserialize(deserializer).map(|mut c| {
                    c.content.version = DidExchangeTypeV1::new_v1_0();
                    c.into()
                })
            }
            DidExchangeTypeV1_0::Complete => Complete::deserialize(deserializer).map(|mut c| {
                c.decorators.version = DidExchangeTypeV1::new_v1_0();
                c.into()
            }),
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Request(v) => {
                MsgWithType::<_, DidExchangeTypeV1_0>::from(v).serialize(serializer)
            }
            Self::Response(v) => MsgWithType::from(v).serialize(serializer),
            Self::ProblemReport(v) => {
                MsgWithType::<_, DidExchangeTypeV1_0>::from(v).serialize(serializer)
            }
            Self::Complete(v) => {
                MsgWithType::<_, DidExchangeTypeV1_0>::from(v).serialize(serializer)
            }
        }
    }
}

transit_to_aries_msg!(ResponseContent: ResponseDecorators, DidExchangeV1_0, DidExchange);

into_msg_with_type!(Request, DidExchangeTypeV1_0, Request);
into_msg_with_type!(Response, DidExchangeTypeV1_0, Response);
into_msg_with_type!(ProblemReport, DidExchangeTypeV1_0, ProblemReport);
into_msg_with_type!(Complete, DidExchangeTypeV1_0, Complete);
