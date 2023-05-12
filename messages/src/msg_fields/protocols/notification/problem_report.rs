use serde::{Deserialize, Serialize};

use crate::{
    msg_fields::protocols::report_problem::{ProblemReportContent, ProblemReportDecorators},
    msg_parts::MsgParts,
};

pub type NotificationProblemReport = MsgParts<NotificationProblemReportContent, ProblemReportDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(transparent)]
pub struct NotificationProblemReportContent(pub ProblemReportContent);

impl NotificationProblemReportContent {
    pub fn new(code: String) -> Self {
        Self(ProblemReportContent::new(code))
    }
}
