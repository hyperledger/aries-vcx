use derive_more::From;
use messages_macros::MessageType;
use strum_macros::{AsRefStr, EnumString};
use transitive::Transitive;

use super::Protocol;
use crate::msg_types::{role::Role, MsgKindType};

#[derive(Copy, Clone, Debug, From, PartialEq, MessageType)]
#[msg_type(protocol = "report-problem")]
pub enum ReportProblemType {
    V1(ReportProblemTypeV1),
}

#[derive(Copy, Clone, Debug, From, PartialEq, Transitive, MessageType)]
#[transitive(into(ReportProblemType, Protocol))]
#[msg_type(major = 1)]
pub enum ReportProblemTypeV1 {
    #[msg_type(minor = 0, roles = "Role::Notified, Role::Notifier")]
    V1_0(MsgKindType<ReportProblemTypeV1_0>),
}

#[derive(Copy, Clone, Debug, AsRefStr, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub enum ReportProblemTypeV1_0 {
    ProblemReport,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_protocol_report_problem() {
        test_utils::test_serde(
            Protocol::from(ReportProblemTypeV1::new_v1_0()),
            json!("https://didcomm.org/report-problem/1.0"),
        )
    }

    #[test]
    fn test_version_resolution_report_problem() {
        test_utils::test_msg_type_resolution(
            "https://didcomm.org/report-problem/1.255",
            ReportProblemTypeV1::new_v1_0(),
        )
    }

    #[test]
    #[should_panic]
    fn test_unsupported_version_report_problem() {
        test_utils::test_serde(
            Protocol::from(ReportProblemTypeV1::new_v1_0()),
            json!("https://didcomm.org/report-problem/2.0"),
        )
    }

    #[test]
    fn test_msg_type_problem_report() {
        test_utils::test_msg_type(
            "https://didcomm.org/report-problem/1.0",
            "problem-report",
            ReportProblemTypeV1::new_v1_0(),
        )
    }
}
