use shared::misc::serde_ignored::SerdeIgnored as NoContent;

use crate::{
    msg_fields::protocols::did_exchange::v1_x::complete::CompleteDecorators, msg_parts::MsgParts,
};

/// Alias type for DIDExchange v1.0 Complete message.
/// Note that since this inherits from the V1.X message, the direct serialization
/// of this message type is not recommended, as version metadata will be lost.
/// Instead, this type should be converted to/from an AriesMessage
pub type Complete = MsgParts<NoContent, CompleteDecorators>;

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
        msg_types::protocols::did_exchange::{DidExchangeTypeV1, DidExchangeTypeV1_0},
    };

    #[test]
    fn test_minimal_complete_message() {
        let thread = make_minimal_thread();

        let expected = json!({
            "~thread": {
                "thid": thread.thid
            }
        });

        let decorators = CompleteDecorators::builder()
            .thread(thread)
            .version(DidExchangeTypeV1::new_v1_0())
            .build();

        test_utils::test_msg(
            NoContent,
            decorators,
            DidExchangeTypeV1_0::Complete,
            expected,
        );
    }

    #[test]
    fn test_extended_complete_message() {
        let decorators = CompleteDecorators {
            thread: make_extended_thread(),
            timing: Some(make_extended_timing()),
            version: DidExchangeTypeV1::new_v1_0(),
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
