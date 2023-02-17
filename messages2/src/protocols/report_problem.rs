use messages_macros::Message;
use serde::{Deserialize, Serialize};

use crate::message_type::message_family::report_problem::{ReportProblem, ReportProblemV1, ReportProblemV1_0};

use super::traits::ConcreteMessage;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "ReportProblem::V1(ReportProblemV1::V1_0(ReportProblemV1_0::ProblemReport))")]
pub struct ProblemReport;
