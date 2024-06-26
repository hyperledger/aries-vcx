use serde::{Deserialize, Serialize};
use shared::maybe_known::MaybeKnown;
use typed_builder::TypedBuilder;

use crate::{
    decorators::{
        attachment::Attachment,
        thread::{Thread, ThreadGoalCode},
        timing::Timing,
    },
    msg_fields::protocols::did_exchange::{
        v1_0::DidExchangeV1_0, v1_1::DidExchangeV1_1, DidExchange,
    },
    msg_parts::MsgParts,
    msg_types::protocols::did_exchange::DidExchangeTypeV1,
    AriesMessage,
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum AnyRequest {
    V1_0(Request),
    V1_1(Request),
}

impl AnyRequest {
    pub fn get_version(&self) -> DidExchangeTypeV1 {
        match self {
            AnyRequest::V1_0(_) => DidExchangeTypeV1::new_v1_0(),
            AnyRequest::V1_1(_) => DidExchangeTypeV1::new_v1_1(),
        }
    }
}

impl AnyRequest {
    pub fn into_inner(self) -> Request {
        match self {
            AnyRequest::V1_0(r) | AnyRequest::V1_1(r) => r,
        }
    }

    pub fn inner(&self) -> &Request {
        match self {
            AnyRequest::V1_0(r) | AnyRequest::V1_1(r) => r,
        }
    }
}

impl From<AnyRequest> for AriesMessage {
    fn from(value: AnyRequest) -> Self {
        match value {
            AnyRequest::V1_0(inner) => DidExchange::V1_0(DidExchangeV1_0::Request(inner)).into(),
            AnyRequest::V1_1(inner) => DidExchange::V1_1(DidExchangeV1_1::Request(inner)).into(),
        }
    }
}
