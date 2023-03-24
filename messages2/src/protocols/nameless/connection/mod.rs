//! Module containing the `connection` protocol messages, as defined in the [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/features/0160-connection-protocol/README.md).

pub mod invitation;
pub mod problem_report;
pub mod request;
pub mod response;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use self::{
    invitation::Invitation,
    problem_report::{ProblemReport, ProblemReportContent, ProblemReportDecorators},
    request::{Request, RequestContent, RequestDecorators},
    response::{Response, ResponseContent, ResponseDecorators},
};
use crate::{
    misc::utils::{self, into_msg_with_type, transit_to_aries_msg},
    msg_types::{
        types::connection::{ConnectionProtocol as ConnectionKind, ConnectionProtocolV1, ConnectionProtocolV1_0},
        MsgWithType,
    },
    protocols::traits::DelayedSerde,
};

#[derive(Clone, Debug, From, PartialEq)]
pub enum Connection {
    Invitation(Invitation),
    Request(Request),
    Response(Response),
    ProblemReport(ProblemReport),
}

impl DelayedSerde for Connection {
    type MsgType<'a> = (ConnectionKind, &'a str);

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (protocol, kind_str) = msg_type;

        let kind = match protocol {
            ConnectionKind::V1(ConnectionProtocolV1::V1_0(kind)) => kind.kind_from_str(kind_str),
        };

        match kind.map_err(D::Error::custom)? {
            ConnectionProtocolV1_0::Invitation => Invitation::deserialize(deserializer).map(From::from),
            ConnectionProtocolV1_0::Request => Request::deserialize(deserializer).map(From::from),
            ConnectionProtocolV1_0::Response => Response::deserialize(deserializer).map(From::from),
            ConnectionProtocolV1_0::ProblemReport => ProblemReport::deserialize(deserializer).map(From::from),
            ConnectionProtocolV1_0::Ed25519Sha512Single => Err(utils::not_standalone_msg::<D>(kind_str)),
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Invitation(v) => MsgWithType::from(v).serialize(serializer),
            Self::Request(v) => MsgWithType::from(v).serialize(serializer),
            Self::Response(v) => MsgWithType::from(v).serialize(serializer),
            Self::ProblemReport(v) => MsgWithType::from(v).serialize(serializer),
        }
    }
}

transit_to_aries_msg!(RequestContent: RequestDecorators, Connection);
transit_to_aries_msg!(ResponseContent: ResponseDecorators, Connection);
transit_to_aries_msg!(ProblemReportContent: ProblemReportDecorators, Connection);

into_msg_with_type!(Invitation, ConnectionProtocolV1_0, Invitation);
into_msg_with_type!(Request, ConnectionProtocolV1_0, Request);
into_msg_with_type!(Response, ConnectionProtocolV1_0, Response);
into_msg_with_type!(ProblemReport, ConnectionProtocolV1_0, ProblemReport);
