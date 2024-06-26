use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{localization::MsgLocalization, thread::Thread, timing::Timing},
    msg_fields::protocols::did_exchange::{
        v1_0::DidExchangeV1_0, v1_1::DidExchangeV1_1, DidExchange,
    },
    msg_parts::MsgParts,
    msg_types::protocols::did_exchange::DidExchangeTypeV1,
    AriesMessage,
};

/// Alias type for the shared DIDExchange v1.X problem report message type.
/// Note the direct serialization of this message type is not recommended,
/// as version metadata will be lost.
/// Instead, this type should be converted to/from an AriesMessage
pub type ProblemReport = MsgParts<ProblemReportContent, ProblemReportDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum AnyProblemReport {
    V1_0(ProblemReport),
    V1_1(ProblemReport),
}

impl AnyProblemReport {
    pub fn get_version(&self) -> DidExchangeTypeV1 {
        match self {
            AnyProblemReport::V1_0(_) => DidExchangeTypeV1::new_v1_0(),
            AnyProblemReport::V1_1(_) => DidExchangeTypeV1::new_v1_1(),
        }
    }
}

impl AnyProblemReport {
    pub fn into_inner(self) -> ProblemReport {
        match self {
            AnyProblemReport::V1_0(r) | AnyProblemReport::V1_1(r) => r,
        }
    }

    pub fn inner(&self) -> &ProblemReport {
        match self {
            AnyProblemReport::V1_0(r) | AnyProblemReport::V1_1(r) => r,
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
