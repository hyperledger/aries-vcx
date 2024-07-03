use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::DidExchangeV1MessageVariant;
use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::did_exchange::{
        v1_0::response::Response as ResponseV1_0,
        v1_1::response::{Response as ResponseV1_1, ResponseContent as ResponseV1_1Content},
    },
};

pub type AnyResponse = DidExchangeV1MessageVariant<ResponseV1_0, ResponseV1_1>;

impl AnyResponse {
    pub fn into_v1_1(self) -> ResponseV1_1 {
        match self {
            AnyResponse::V1_0(r) => r.into_v1_1(),
            AnyResponse::V1_1(r) => r,
        }
    }
}

impl ResponseV1_0 {
    pub fn into_v1_1(self) -> ResponseV1_1 {
        ResponseV1_1 {
            id: self.id,
            decorators: self.decorators,
            content: ResponseV1_1Content {
                did: self.content.did,
                did_doc: self.content.did_doc,
                did_rotate: None,
            },
        }
    }
}

impl From<ResponseV1_0> for AnyResponse {
    fn from(value: ResponseV1_0) -> Self {
        Self::V1_0(value)
    }
}

impl From<ResponseV1_1> for AnyResponse {
    fn from(value: ResponseV1_1) -> Self {
        Self::V1_1(value)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, TypedBuilder)]
pub struct ResponseDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
