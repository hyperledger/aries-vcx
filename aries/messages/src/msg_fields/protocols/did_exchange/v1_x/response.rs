use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::did_exchange::{
        v1_0::{response::Response as ResponseV1_0, DidExchangeV1_0},
        v1_1::{
            response::{Response as ResponseV1_1, ResponseContent as ResponseV1_1Content},
            DidExchangeV1_1,
        },
        DidExchange,
    },
    msg_types::protocols::did_exchange::DidExchangeTypeV1,
    AriesMessage,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, derive_more::From)]
#[serde(untagged)]
pub enum AnyResponse {
    V1_0(ResponseV1_0),
    V1_1(ResponseV1_1),
}

impl AnyResponse {
    pub fn get_version(&self) -> DidExchangeTypeV1 {
        match self {
            AnyResponse::V1_0(_) => DidExchangeTypeV1::new_v1_0(),
            AnyResponse::V1_1(_) => DidExchangeTypeV1::new_v1_1(),
        }
    }

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

impl From<AnyResponse> for AriesMessage {
    fn from(value: AnyResponse) -> Self {
        match value {
            AnyResponse::V1_0(inner) => DidExchange::V1_0(DidExchangeV1_0::Response(inner)).into(),
            AnyResponse::V1_1(inner) => DidExchange::V1_1(DidExchangeV1_1::Response(inner)).into(),
        }
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
