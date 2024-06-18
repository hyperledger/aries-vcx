use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use shared::misc::serde_ignored::SerdeIgnored as NoContent;
use typed_builder::TypedBuilder;

use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::did_exchange::{
        v1_0::{complete::Complete as CompleteV1_0, DidExchangeV1_0},
        v1_1::{
            complete::{Complete as CompleteV1_1, CompleteDecoratorsV1_1},
            DidExchangeV1_1,
        },
        DidExchange,
    },
    msg_parts::MsgParts,
    msg_types::protocols::did_exchange::DidExchangeTypeV1,
    AriesMessage,
};

pub type Complete<MinorVer> = MsgParts<NoContent, CompleteDecorators<MinorVer>>;

// TODO: Pthid is mandatory in this case!
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct CompleteDecorators<MinorVer> {
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
    #[builder(default, setter(skip))]
    #[serde(skip)]
    pub(crate) _marker: PhantomData<MinorVer>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, derive_more::From)]
#[serde(untagged)]
pub enum AnyComplete {
    V1_0(CompleteV1_0),
    V1_1(CompleteV1_1),
}

impl AnyComplete {
    pub fn get_version_marker(&self) -> DidExchangeTypeV1 {
        match self {
            AnyComplete::V1_0(_) => DidExchangeTypeV1::new_v1_0(),
            AnyComplete::V1_1(_) => DidExchangeTypeV1::new_v1_1(),
        }
    }

    pub fn into_v1_1(self) -> CompleteV1_1 {
        match self {
            AnyComplete::V1_0(r) => r.into_v1_1(),
            AnyComplete::V1_1(r) => r,
        }
    }
}

impl CompleteV1_0 {
    pub fn into_v1_1(self) -> CompleteV1_1 {
        CompleteV1_1 {
            id: self.id,
            content: self.content,
            decorators: CompleteDecoratorsV1_1 {
                thread: self.decorators.thread,
                timing: self.decorators.timing,
                _marker: PhantomData,
            },
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

// #[cfg(test)]
// #[allow(clippy::unwrap_used)]
// #[allow(clippy::field_reassign_with_default)]
// mod tests {
//     use serde_json::json;

//     use super::*;
//     use crate::{
//         decorators::{
//             thread::tests::{make_extended_thread, make_minimal_thread},
//             timing::tests::make_extended_timing,
//         },
//         misc::test_utils,
//         msg_types::protocols::did_exchange::DidExchangeTypeV1_0,
//     };

//     #[test]
//     fn test_minimal_complete_message() {
//         let thread = make_minimal_thread();

//         let expected = json!({
//             "~thread": {
//                 "thid": thread.thid
//             }
//         });

//         let decorators = CompleteDecorators {
//             thread,
//             timing: None,
//             _marker: PhantomData::<()>,
//         };

//         test_utils::test_msg(
//             NoContent,
//             decorators,
//             DidExchangeTypeV1_0::Complete,
//             expected,
//         );
//     }

//     #[test]
//     fn test_extended_complete_message() {
//         let decorators = CompleteDecorators {
//             thread: make_extended_thread(),
//             timing: Some(make_extended_timing()),
//             _marker: PhantomData::<()>,
//         };

//         let expected = json!({
//             "~thread": serde_json::to_value(make_extended_thread()).unwrap(),
//             "~timing": serde_json::to_value(make_extended_timing()).unwrap()
//         });

//         test_utils::test_msg(
//             NoContent,
//             decorators,
//             DidExchangeTypeV1_0::Complete,
//             expected,
//         );
//     }
// }
