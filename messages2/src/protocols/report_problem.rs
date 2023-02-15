use serde::{Deserialize, Serialize};

use crate::message_type::message_family::{
    report_problem::{ReportProblem, ReportProblemV1, ReportProblemV1_0},
    traits::ConcreteMessage,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProblemReport;

impl ConcreteMessage for ProblemReport {
    type Kind = ReportProblem;

    fn kind() -> Self::Kind {
        Self::Kind::V1(ReportProblemV1::V1_0(ReportProblemV1_0::ProblemReport))
    }
}
