use serde::{Deserialize, Serialize};
use shared::maybe_known::MaybeKnown;
use typed_builder::TypedBuilder;

use super::DidExchangeV1MessageVariant;
use crate::{
    decorators::{
        attachment::Attachment,
        thread::{Thread, ThreadGoalCode},
        timing::Timing,
    },
    msg_parts::MsgParts,
};

/// Alias type for the shared DIDExchange v1.X request message type.
/// Note the direct serialization of this message type is not recommended,
/// as it will be indistinguisable between V1.1 & V1.0.
/// Instead, this type should be converted to/from an AriesMessage
pub type Request = MsgParts<RequestContent, RequestDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct RequestContent {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<MaybeKnown<ThreadGoalCode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    pub did: String, // TODO: Use Did
    #[serde(rename = "did_doc~attach", skip_serializing_if = "Option::is_none")]
    pub did_doc: Option<Attachment>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct RequestDecorators {
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

pub type AnyRequest = DidExchangeV1MessageVariant<Request, Request>;
