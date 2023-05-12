//! Module containing the `acks` messages, as defined in the [RFC](<https://github.com/hyperledger/aries-rfcs/blob/main/features/0015-acks/README.md>).

pub mod ack;
pub mod problem_report;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use self::{
    ack::{Ack, AckContent, AckDecorators},
    problem_report::{NotificationProblemReport, NotificationProblemReportContent},
};
use crate::{
    misc::utils::{into_msg_with_type, transit_to_aries_msg},
    msg_fields::traits::DelayedSerde,
    msg_types::{
        notification::{NotificationType, NotificationTypeV1, NotificationTypeV1_0},
        MsgWithType,
    },
};

use super::report_problem::ProblemReportDecorators;

#[derive(Clone, Debug, From, PartialEq)]
pub enum Notification {
    Ack(Ack),
    ProblemReport(NotificationProblemReport),
}

impl DelayedSerde for Notification {
    type MsgType<'a> = (NotificationType, &'a str);

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (protocol, kind_str) = msg_type;

        let kind = match protocol {
            NotificationType::V1(NotificationTypeV1::V1_0(kind)) => kind.kind_from_str(kind_str),
        };

        match kind.map_err(D::Error::custom)? {
            NotificationTypeV1_0::Ack => Ack::deserialize(deserializer).map(From::from),
            NotificationTypeV1_0::ProblemReport => NotificationProblemReport::deserialize(deserializer).map(From::from),
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Ack(v) => MsgWithType::from(v).serialize(serializer),
            Self::ProblemReport(v) => MsgWithType::from(v).serialize(serializer),
        }
    }
}

transit_to_aries_msg!(AckContent: AckDecorators, Notification);
transit_to_aries_msg!(NotificationProblemReportContent: ProblemReportDecorators, Notification);

into_msg_with_type!(Ack, NotificationTypeV1_0, Ack);
into_msg_with_type!(NotificationProblemReport, NotificationTypeV1_0, ProblemReport);
