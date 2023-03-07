pub mod invitation;
pub mod problem_report;
pub mod request;
pub mod response;

use derive_more::From;
use serde::{Deserializer, Serializer};

use crate::{
    composite_message::{transit_to_aries_msg, Message},
    delayed_serde::DelayedSerde,
    message_type::message_family::connection::{Connection as ConnectionKind, ConnectionV1, ConnectionV1_0},
    utils,
};

use self::{
    invitation::Invitation,
    problem_report::{ProblemReportContent, ProblemReportDecorators},
    request::{RequestContent, RequestDecorators},
    response::{ResponseContent, ResponseDecorators},
};

pub use self::{problem_report::ProblemReport, request::Request, response::Response};

pub use invitation::CompleteInvitationContent;

#[derive(Clone, Debug, From)]
pub enum Connection {
    Invitation(Invitation),
    Request(Request),
    Response(Response),
    ProblemReport(ProblemReport),
}

impl DelayedSerde for Connection {
    type MsgType = ConnectionKind;

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ConnectionKind::V1(major) = msg_type;
        let ConnectionV1::V1_0(minor) = major;

        match minor {
            ConnectionV1_0::Invitation => Invitation::delayed_deserialize(minor, deserializer).map(From::from),
            ConnectionV1_0::Request => Request::delayed_deserialize(minor, deserializer).map(From::from),
            ConnectionV1_0::Response => Response::delayed_deserialize(minor, deserializer).map(From::from),
            ConnectionV1_0::ProblemReport => ProblemReport::delayed_deserialize(minor, deserializer).map(From::from),
            ConnectionV1_0::Ed25519Sha512Single => Err(utils::not_standalone_msg::<D>(minor.as_ref())),
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Invitation(v) => v.delayed_serialize(serializer),
            Self::Request(v) => v.delayed_serialize(serializer),
            Self::Response(v) => v.delayed_serialize(serializer),
            Self::ProblemReport(v) => v.delayed_serialize(serializer),
        }
    }
}

transit_to_aries_msg!(RequestContent: RequestDecorators, Connection);
transit_to_aries_msg!(ResponseContent: ResponseDecorators, Connection);
transit_to_aries_msg!(ProblemReportContent: ProblemReportDecorators, Connection);
