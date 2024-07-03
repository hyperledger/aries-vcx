use serde::{Deserialize, Serialize};
use shared::misc::serde_ignored::SerdeIgnored as NoContent;
use typed_builder::TypedBuilder;

use super::DidExchangeV1MessageVariant;
use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_parts::MsgParts,
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

pub type AnyComplete = DidExchangeV1MessageVariant<Complete, Complete>;
