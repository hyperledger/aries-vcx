use derive_more::From;
use messages_macros::{MessageType, TransitiveFrom};
use strum_macros::{AsRefStr, EnumString};

use crate::error::{MsgTypeError, MsgTypeResult};

use super::{
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    MessageFamily,
};

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[semver(family = "report-problem")]
pub enum ReportProblem {
    V1(ReportProblemV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom, MessageType)]
#[transitive(ReportProblem, MessageFamily)]
#[semver(major = 1)]
pub enum ReportProblemV1 {
    V1_0(ReportProblemV1_0),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveFrom, MessageType)]
#[transitive(ReportProblemV1, ReportProblem, MessageFamily)]
#[strum(serialize_all = "kebab-case")]
#[semver(minor = 0)]
pub enum ReportProblemV1_0 {
    ProblemReport,
}
