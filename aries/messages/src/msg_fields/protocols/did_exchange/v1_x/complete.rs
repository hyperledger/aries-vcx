use serde::{Deserialize, Serialize};
use shared::misc::serde_ignored::SerdeIgnored as NoContent;
use typed_builder::TypedBuilder;

use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::did_exchange::{
        v1_0::DidExchangeV1_0, v1_1::DidExchangeV1_1, DidExchange,
    },
    msg_parts::MsgParts,
    msg_types::protocols::did_exchange::DidExchangeTypeV1,
    AriesMessage,
};

/// Alias type for the shared DIDExchange v1.X complete message type.
/// Note the direct serialization of this message type is not recommended,
/// as it will be indistinguisable between V1.1 & V1.0.
/// Instead, this type should be converted to/from an AriesMessage
pub type Complete = MsgParts<NoContent, CompleteDecorators>;

// TODO: Pthid is mandatory in this case!
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct CompleteDecorators {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum AnyComplete {
    V1_0(Complete),
    V1_1(Complete),
}

impl AnyComplete {
    pub fn get_version(&self) -> DidExchangeTypeV1 {
        match self {
            AnyComplete::V1_0(_) => DidExchangeTypeV1::new_v1_0(),
            AnyComplete::V1_1(_) => DidExchangeTypeV1::new_v1_1(),
        }
    }
}

impl AnyComplete {
    pub fn into_inner(self) -> Complete {
        match self {
            AnyComplete::V1_0(r) | AnyComplete::V1_1(r) => r,
        }
    }

    pub fn inner(&self) -> &Complete {
        match self {
            AnyComplete::V1_0(r) | AnyComplete::V1_1(r) => r,
        }
    }
}

impl From<AnyComplete> for AriesMessage {
    fn from(value: AnyComplete) -> Self {
        match value {
            AnyComplete::V1_0(inner) => DidExchange::V1_0(DidExchangeV1_0::Complete(inner)).into(),
            AnyComplete::V1_1(inner) => DidExchange::V1_1(DidExchangeV1_1::Complete(inner)).into(),
        }
    }
}
