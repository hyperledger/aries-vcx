use serde::{Deserialize, Serialize};

use crate::{
    msg_fields::protocols::report_problem::{ProblemReportContent, ProblemReportDecorators},
    msg_parts::MsgParts,
};

pub type PresentProofProblemReport = MsgParts<PresentProofProblemReportContent, ProblemReportDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub struct PresentProofProblemReportContent(pub ProblemReportContent);

impl PresentProofProblemReportContent {
    pub fn new(code: String) -> Self {
        Self(ProblemReportContent::new(code))
    }
}
