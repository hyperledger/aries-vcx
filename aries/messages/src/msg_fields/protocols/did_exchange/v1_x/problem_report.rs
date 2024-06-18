use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{localization::MsgLocalization, thread::Thread, timing::Timing},
    msg_fields::protocols::did_exchange::{
        v1_0::{problem_report::ProblemReport as ProblemReportV1_0, DidExchangeV1_0},
        v1_1::{
            problem_report::{ProblemReport as ProblemReportV1_1, ProblemReportContentV1_1},
            DidExchangeV1_1,
        },
        DidExchange,
    },
    msg_parts::MsgParts,
    msg_types::protocols::did_exchange::DidExchangeTypeV1,
    AriesMessage,
};

pub type ProblemReport<MinorVer> =
    MsgParts<ProblemReportContent<MinorVer>, ProblemReportDecorators>;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, derive_more::From)]
#[serde(untagged)]
pub enum AnyProblemReport {
    V1_0(ProblemReportV1_0),
    V1_1(ProblemReportV1_1),
}

impl AnyProblemReport {
    pub fn get_version_marker(&self) -> DidExchangeTypeV1 {
        match self {
            AnyProblemReport::V1_0(_) => DidExchangeTypeV1::new_v1_0(),
            AnyProblemReport::V1_1(_) => DidExchangeTypeV1::new_v1_1(),
        }
    }

    pub fn into_v1_1(self) -> ProblemReportV1_1 {
        match self {
            AnyProblemReport::V1_0(r) => r.into_v1_1(),
            AnyProblemReport::V1_1(r) => r,
        }
    }
}

impl ProblemReportV1_0 {
    pub fn into_v1_1(self) -> ProblemReportV1_1 {
        ProblemReportV1_1 {
            id: self.id,
            content: ProblemReportContentV1_1 {
                problem_code: self.content.problem_code,
                explain: self.content.explain,
                _marker: PhantomData,
            },
            decorators: self.decorators,
        }
    }
}

impl From<AnyProblemReport> for AriesMessage {
    fn from(value: AnyProblemReport) -> Self {
        match value {
            AnyProblemReport::V1_0(inner) => {
                DidExchange::V1_0(DidExchangeV1_0::ProblemReport(inner)).into()
            }
            AnyProblemReport::V1_1(inner) => {
                DidExchange::V1_1(DidExchangeV1_1::ProblemReport(inner)).into()
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct ProblemReportContent<MinorVer> {
    #[serde(rename = "problem-code")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_code: Option<ProblemCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explain: Option<String>,
    #[builder(default)]
    #[serde(skip)]
    pub(crate) _marker: PhantomData<MinorVer>,
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

// #[cfg(test)]
// #[allow(clippy::unwrap_used)]
// #[allow(clippy::field_reassign_with_default)]
// mod tests {
//     use serde_json::json;

//     use super::*;
//     use crate::{
//         decorators::{
//             localization::tests::make_extended_msg_localization,
//             thread::tests::make_extended_thread, timing::tests::make_extended_timing,
//         },
//         misc::test_utils,
//         msg_types::protocols::did_exchange::DidExchangeTypeV1_0,
//     };

//     #[test]
//     fn test_minimal_conn_problem_report() {
//         let content = ProblemReportContent::<()>::default();

//         let decorators = ProblemReportDecorators::new(make_extended_thread());

//         let expected = json!({
//             "~thread": decorators.thread
//         });

//         test_utils::test_msg(
//             content,
//             decorators,
//             DidExchangeTypeV1_0::ProblemReport,
//             expected,
//         );
//     }

//     #[test]
//     fn test_extended_conn_problem_report() {
//         let mut content = ProblemReportContent::<()>::default();
//         content.problem_code = Some(ProblemCode::RequestNotAccepted);
//         content.explain = Some("test_conn_problem_report_explain".to_owned());

//         let mut decorators = ProblemReportDecorators::new(make_extended_thread());
//         decorators.timing = Some(make_extended_timing());
//         decorators.localization = Some(make_extended_msg_localization());

//         let expected = json!({
//             "problem-code": content.problem_code,
//             "explain": content.explain,
//             "~thread": decorators.thread,
//             "~timing": decorators.timing,
//             "~l10n": decorators.localization
//         });

//         test_utils::test_msg(
//             content,
//             decorators,
//             DidExchangeTypeV1_0::ProblemReport,
//             expected,
//         );
//     }
// }
