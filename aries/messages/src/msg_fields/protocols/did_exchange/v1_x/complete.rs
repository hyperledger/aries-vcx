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
/// as version metadata will be lost.
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
    #[serde(skip, default = "DidExchangeTypeV1::new_v1_1")]
    pub(crate) version: DidExchangeTypeV1,
}

impl Complete {
    pub fn get_version(&self) -> DidExchangeTypeV1 {
        self.decorators.version
    }
}

impl From<Complete> for AriesMessage {
    fn from(value: Complete) -> Self {
        match value.get_version() {
            DidExchangeTypeV1::V1_0(_) => {
                DidExchange::V1_0(DidExchangeV1_0::Complete(value)).into()
            }
            DidExchangeTypeV1::V1_1(_) => {
                DidExchange::V1_1(DidExchangeV1_1::Complete(value)).into()
            }
        }
    }
}
