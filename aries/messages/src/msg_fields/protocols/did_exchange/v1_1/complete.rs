use serde::{Deserialize, Serialize};
use shared::misc::serde_ignored::SerdeIgnored as NoContent;
use typed_builder::TypedBuilder;

use crate::{
    decorators::{thread::Thread, timing::Timing},
    msg_parts::MsgParts,
};

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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{
            thread::tests::{make_extended_thread, make_minimal_thread},
            timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_types::protocols::did_exchange::DidExchangeTypeV1_1,
    };

    #[test]
    fn test_minimal_complete_message() {
        let thread = make_minimal_thread();

        let expected = json!({
            "~thread": {
                "thid": thread.thid
            }
        });

        let decorators = CompleteDecorators {
            thread,
            timing: None,
        };

        test_utils::test_msg(
            NoContent,
            decorators,
            DidExchangeTypeV1_1::Complete,
            expected,
        );
    }

    #[test]
    fn test_extended_complete_message() {
        let decorators = CompleteDecorators {
            thread: make_extended_thread(),
            timing: Some(make_extended_timing()),
        };

        let expected = json!({
            "~thread": serde_json::to_value(make_extended_thread()).unwrap(),
            "~timing": serde_json::to_value(make_extended_timing()).unwrap()
        });

        test_utils::test_msg(
            NoContent,
            decorators,
            DidExchangeTypeV1_1::Complete,
            expected,
        );
    }
}
