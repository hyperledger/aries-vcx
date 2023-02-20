use messages_macros::Message;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{Localization, Thread, Timing},
    message_type::message_family::connection::ConnectionV1_0,
};

use crate::protocols::traits::ConcreteMessage;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "ConnectionV1_0::ProblemReport")]
pub struct ProblemReport {
    #[serde(rename = "@id")]
    pub id: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProblemCode {
    RequestNotAccepted,
    RequestProcessingError,
    ResponseNotAccepted,
    ResponseProcessingError,
}
