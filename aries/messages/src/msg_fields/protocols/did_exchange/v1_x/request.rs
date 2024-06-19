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
/// as version metadata will be lost.
/// Instead, this type should be converted to/from an AriesMessage
pub type Request = MsgParts<RequestContent, RequestDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct RequestContent {
    pub label: String,
    pub goal_code: Option<MaybeKnown<ThreadGoalCode>>,
    pub goal: Option<String>,
    pub did: String, // TODO: Use Did
    #[serde(rename = "did_doc~attach")]
    pub did_doc: Option<Attachment>,
    #[serde(skip, default = "DidExchangeTypeV1::new_v1_1")]
    pub(crate) version: DidExchangeTypeV1,
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

impl Request {
    pub fn get_version(&self) -> DidExchangeTypeV1 {
        self.content.version
    }
}

impl From<Request> for AriesMessage {
    fn from(value: Request) -> Self {
        match value.get_version() {
            DidExchangeTypeV1::V1_0(_) => DidExchange::V1_0(DidExchangeV1_0::Request(value)).into(),
            DidExchangeTypeV1::V1_1(_) => DidExchange::V1_1(DidExchangeV1_1::Request(value)).into(),
        }
    }
}
