use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::TransitiveFrom;

use crate::{
    error::{MsgTypeError, MsgTypeResult},
    message_type::actor::Actor,
    message_type::MessageType,
};

use super::{
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    MessageFamily,
};

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(MessageFamily, MessageType))]
#[semver(family = "report-problem")]
pub enum ReportProblem {
    V1(ReportProblemV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(ReportProblem, MessageFamily, MessageType)))]
#[semver(major = 1, parent = "ReportProblem", actors("Actor::Notified", "Actor::Notifier"))]
pub enum ReportProblemV1 {
    V1_0(ReportProblemV1_0),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveFrom, MessageType)]
#[transitive(into(all(ReportProblemV1, ReportProblem, MessageFamily, MessageType)))]
#[strum(serialize_all = "kebab-case")]
#[semver(minor = 0, parent = "ReportProblemV1")]
pub enum ReportProblemV1_0 {
    ProblemReport,
}
