use serde::{Deserialize, Serialize};

use crate::{
    msg_fields::protocols::report_problem::{ProblemReportContent, ProblemReportDecorators},
    msg_parts::MsgParts,
};

pub type CredIssuanceProblemReport = MsgParts<CredIssuanceProblemReportContent, ProblemReportDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub struct CredIssuanceProblemReportContent(pub ProblemReportContent);

impl CredIssuanceProblemReportContent {
    pub fn new(code: String) -> Self {
        Self(ProblemReportContent::new(code))
    }
}
