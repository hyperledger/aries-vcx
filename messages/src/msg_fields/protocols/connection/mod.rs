//! Module containing the `connection` protocol messages, as defined in the [RFC](<https://github.com/hyperledger/aries-rfcs/blob/main/features/0160-connection-protocol/README.md>).

pub mod invitation;
pub mod problem_report;
pub mod request;
pub mod response;

use derive_more::From;
use did_doc::schema::did_doc::DidDocument;
use did_resolver_sov::resolution::ExtraFieldsSov;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use self::{
    invitation::Invitation,
    problem_report::{ProblemReport, ProblemReportContent, ProblemReportDecorators},
    request::{Request, RequestContent, RequestDecorators},
    response::{Response, ResponseContent, ResponseDecorators},
};
use crate::{
    misc::utils::{into_msg_with_type, transit_to_aries_msg},
    msg_fields::traits::DelayedSerde,
    msg_types::{
        protocols::connection::{ConnectionType as ConnectionKind, ConnectionTypeV1, ConnectionTypeV1_0},
        MsgWithType,
    },
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
            ConnectionKind::V1(ConnectionTypeV1::V1_0(kind)) => kind.kind_from_str(kind_str),
        };

        match kind.map_err(D::Error::custom)? {
            ConnectionTypeV1_0::Invitation => Invitation::deserialize(deserializer).map(From::from),
            ConnectionTypeV1_0::Request => Request::deserialize(deserializer).map(From::from),
            ConnectionTypeV1_0::Response => Response::deserialize(deserializer).map(From::from),
            ConnectionTypeV1_0::ProblemReport => ProblemReport::deserialize(deserializer).map(From::from),
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ConnectionData {
    #[serde(rename = "DID")]
    pub did: String,
    #[serde(rename = "DIDDoc")]
    pub did_doc: DidDocument<ExtraFieldsSov>,
}

impl ConnectionData {
    pub fn new(did: String, did_doc: DidDocument<ExtraFieldsSov>) -> Self {
        Self { did, did_doc }
    }
}

transit_to_aries_msg!(RequestContent: RequestDecorators, Connection);
transit_to_aries_msg!(ResponseContent: ResponseDecorators, Connection);
transit_to_aries_msg!(ProblemReportContent: ProblemReportDecorators, Connection);

into_msg_with_type!(Invitation, ConnectionTypeV1_0, Invitation);
into_msg_with_type!(Request, ConnectionTypeV1_0, Request);
into_msg_with_type!(Response, ConnectionTypeV1_0, Response);
into_msg_with_type!(ProblemReport, ConnectionTypeV1_0, ProblemReport);
