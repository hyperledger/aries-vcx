use crate::{
    a2a::{A2AMessage, MessageId},
    concepts::{localization::Localization, thread::Thread, timing::Timing},
    timing_optional,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProblemReport {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "problem-code")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_code: Option<ProblemCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explain: Option<String>,
    #[serde(rename = "~l10n")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub localization: Option<Localization>,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

threadlike!(ProblemReport);
a2a_message!(ProblemReport, ConnectionProblemReport);
timing_optional!(ProblemReport);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProblemCode {
    Empty,
    #[serde(rename = "request_not_accepted")]
    RequestNotAccepted,
    #[serde(rename = "request_processing_error")]
    RequestProcessingError,
    #[serde(rename = "response_not_accepted")]
    ResponseNotAccepted,
    #[serde(rename = "response_processing_error")]
    ResponseProcessingError,
}

impl ProblemReport {
    pub fn create() -> Self {
        ProblemReport::default()
    }

    pub fn set_problem_code(mut self, problem_code: ProblemCode) -> ProblemReport {
        self.problem_code = Some(problem_code);
        self
    }

    pub fn set_explain(mut self, explain: String) -> ProblemReport {
        self.explain = Some(explain);
        self
    }
}

impl Default for ProblemCode {
    fn default() -> ProblemCode {
        ProblemCode::Empty
    }
}

#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;
    use crate::protocols::connection::response::test_utils::*;

    fn _problem_code() -> ProblemCode {
        ProblemCode::ResponseProcessingError
    }

    fn _explain() -> String {
        String::from("test explanation")
    }

    pub fn _problem_report() -> ProblemReport {
        ProblemReport {
            id: MessageId::id(),
            problem_code: Some(_problem_code()),
            explain: Some(_explain()),
            localization: None,
            thread: _thread(),
            timing: None,
        }
    }

    #[test]
    fn test_problem_report_build_works() {
        let report: ProblemReport = ProblemReport::default()
            .set_problem_code(_problem_code())
            .set_explain(_explain())
            .set_thread_id(&_thread_id());

        assert_eq!(_problem_report(), report);
    }
}
