use shared::misc::serde_ignored::SerdeIgnored as NoContent;

use crate::{
    msg_fields::protocols::did_exchange::v1_x::complete::CompleteDecorators,
    msg_parts::MsgParts,
    msg_types::{protocols::did_exchange::DidExchangeTypeV1_0, MsgKindType},
};

pub type CompleteDecoratorsV1_0 = CompleteDecorators<MsgKindType<DidExchangeTypeV1_0>>;
pub type Complete = MsgParts<NoContent, CompleteDecoratorsV1_0>;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use std::marker::PhantomData;

    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{
            thread::tests::{make_extended_thread, make_minimal_thread},
            timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_types::protocols::did_exchange::DidExchangeTypeV1_0,
    };

    #[test]
    fn test_minimal_complete_message() {
        let thread = make_minimal_thread();

        let expected = json!({
            "~thread": {
                "thid": thread.thid
            }
        });

        let decorators = CompleteDecoratorsV1_0::builder().thread(thread).build();

        test_utils::test_msg(
            NoContent,
            decorators,
            DidExchangeTypeV1_0::Complete,
            expected,
        );
    }

    #[test]
    fn test_extended_complete_message() {
        let decorators = CompleteDecoratorsV1_0 {
            thread: make_extended_thread(),
            timing: Some(make_extended_timing()),
            _marker: PhantomData,
        };

        let expected = json!({
            "~thread": serde_json::to_value(make_extended_thread()).unwrap(),
            "~timing": serde_json::to_value(make_extended_timing()).unwrap()
        });

        test_utils::test_msg(
            NoContent,
            decorators,
            DidExchangeTypeV1_0::Complete,
            expected,
        );
    }
}
