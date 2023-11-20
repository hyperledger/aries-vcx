use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{localization::MsgLocalization, thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

pub type ProblemReport = MsgParts<ProblemReportContent, ProblemReportDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct ProblemReportContent {
    #[serde(rename = "problem-code")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_code: Option<ProblemCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProblemCode {
    RequestNotAccepted,
    RequestProcessingError,
    ResponseNotAccepted,
    ResponseProcessingError,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct ProblemReportDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~l10n")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub localization: Option<MsgLocalization>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

impl ProblemReportDecorators {
    pub fn new(thread: Thread) -> Self {
        Self {
            thread,
            localization: None,
            timing: None,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{
            localization::tests::make_extended_msg_localization,
            thread::tests::make_extended_thread, timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_types::protocols::did_exchange::DidExchangeTypeV1_0,
    };

    #[test]
    fn test_minimal_conn_problem_report() {
        let content = ProblemReportContent::default();

        let decorators = ProblemReportDecorators::new(make_extended_thread());

        let expected = json!({
            "~thread": decorators.thread
        });

        test_utils::test_msg(
            content,
            decorators,
            DidExchangeTypeV1_0::ProblemReport,
            expected,
        );
    }

    #[test]
    fn test_extended_conn_problem_report() {
        let mut content = ProblemReportContent::default();
        content.problem_code = Some(ProblemCode::RequestNotAccepted);
        content.explain = Some("test_conn_problem_report_explain".to_owned());

        let mut decorators = ProblemReportDecorators::new(make_extended_thread());
        decorators.timing = Some(make_extended_timing());
        decorators.localization = Some(make_extended_msg_localization());

        let expected = json!({
            "problem-code": content.problem_code,
            "explain": content.explain,
            "~thread": decorators.thread,
            "~timing": decorators.timing,
            "~l10n": decorators.localization
        });

        test_utils::test_msg(
            content,
            decorators,
            DidExchangeTypeV1_0::ProblemReport,
            expected,
        );
    }
}
