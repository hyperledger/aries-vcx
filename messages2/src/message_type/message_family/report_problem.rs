use derive_more::From;
use messages_macros::TransitiveFrom;
use strum_macros::{AsRefStr, EnumString};

use crate::error::{MsgTypeError, MsgTypeResult};

use super::{
    traits::{ResolveMajorVersion, ResolveMinorVersion, ResolveMsgKind},
    MessageFamily,
};

#[derive(Copy, Clone, Debug, From, PartialEq)]
pub enum ReportProblem {
    V1(ReportProblemV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, TransitiveFrom)]
#[transitive(ReportProblem, MessageFamily)]
pub enum ReportProblemV1 {
    V1_0(ReportProblemV1_0),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq, TransitiveFrom)]
#[transitive(ReportProblemV1, ReportProblem, MessageFamily)]
#[strum(serialize_all = "kebab-case")]
pub enum ReportProblemV1_0 {
    ProblemReport,
}

impl ResolveMsgKind for ReportProblemV1_0 {
    const MINOR: u8 = 0;
}

impl ResolveMinorVersion for ReportProblemV1 {
    const MAJOR: u8 = 1;

    fn resolve_minor_ver(minor: u8, kind: &str) -> MsgTypeResult<Self> {
        match minor {
            ReportProblemV1_0::MINOR => Ok(Self::V1_0(ReportProblemV1_0::resolve_kind(kind)?)),
            _ => Err(MsgTypeError::minor_ver_err(minor)),
        }
    }

    fn as_full_ver_parts(&self) -> (u8, u8, &str) {
        let (minor, kind) = match self {
            Self::V1_0(v) => v.as_minor_ver_parts(),
        };
        (Self::MAJOR, minor, kind)
    }
}

impl ResolveMajorVersion for ReportProblem {
    const FAMILY: &'static str = "report-problem";

    fn resolve_major_ver(major: u8, minor: u8, kind: &str) -> MsgTypeResult<Self> {
        match major {
            ReportProblemV1::MAJOR => Ok(Self::V1(ReportProblemV1::resolve_minor_ver(minor, kind)?)),
            _ => Err(MsgTypeError::major_ver_err(major)),
        }
    }

    fn as_msg_type_parts(&self) -> (&str, u8, u8, &str) {
        let (major, minor, kind) = match self {
            Self::V1(v) => v.as_full_ver_parts(),
        };

        (Self::FAMILY, major, minor, kind)
    }
}
