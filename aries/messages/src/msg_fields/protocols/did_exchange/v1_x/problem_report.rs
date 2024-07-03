use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::DidExchangeV1MessageVariant;
use crate::{
    decorators::{localization::MsgLocalization, thread::Thread, timing::Timing},
    msg_parts::MsgParts,
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

pub type AnyProblemReport = DidExchangeV1MessageVariant<ProblemReport, ProblemReport>;
