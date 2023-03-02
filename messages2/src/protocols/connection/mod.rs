mod invitation;
mod problem_report;
mod request;
mod response;

use derive_more::From;
use serde::{Deserializer, Serializer};

use crate::{
    composite_message::Message,
    delayed_serde::DelayedSerde,
    message_type::message_family::connection::{Connection as ConnectionKind, ConnectionV1, ConnectionV1_0},
    utils,
};

use self::{
    invitation::Invitation,
    problem_report::{ProblemReport, ProblemReportDecorators},
    request::{Request, RequestDecorators},
    response::{Response, ResponseDecorators},
};

pub use invitation::CompleteInvitation;

#[derive(Clone, Debug, From)]
pub enum Connection {
    Invitation(Invitation),
    Request(Message<Request, RequestDecorators>),
    Response(Message<Response, ResponseDecorators>),
    ProblemReport(Message<ProblemReport, ProblemReportDecorators>),
}

impl DelayedSerde for Connection {
    type MsgType = ConnectionKind;

    fn delayed_deserialize<'de, D>(seg: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ConnectionKind::V1(major) = seg;
        let ConnectionV1::V1_0(minor) = major;

        match minor {
            ConnectionV1_0::Invitation => Invitation::delayed_deserialize(minor, deserializer).map(From::from),
            ConnectionV1_0::Request => {
                Message::<Request, RequestDecorators>::delayed_deserialize(minor, deserializer).map(From::from)
            }
            ConnectionV1_0::Response => {
                Message::<Response, ResponseDecorators>::delayed_deserialize(minor, deserializer).map(From::from)
            }
            ConnectionV1_0::ProblemReport => {
                Message::<ProblemReport, ProblemReportDecorators>::delayed_deserialize(minor, deserializer)
                    .map(From::from)
            }
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
