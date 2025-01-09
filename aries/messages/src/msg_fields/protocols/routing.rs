//! Module containing the `mediator and relays` messages, as defined in the [RFC](<https://github.com/hyperledger/aries-rfcs/blob/main/concepts/0046-mediators-and-relays/README.md>).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use typed_builder::TypedBuilder;

use crate::{
    misc::utils::into_msg_with_type, msg_parts::MsgParts,
    msg_types::protocols::routing::RoutingTypeV1_0,
};

pub type Forward = MsgParts<ForwardContent>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct ForwardContent {
    pub to: String,
    pub msg: Value,
}

into_msg_with_type!(Forward, RoutingTypeV1_0, Forward);

#[cfg(test)]
mod tests {
    use serde_json::json;
    // Bind `shared::misc::serde_ignored::SerdeIgnored` type as `NoDecorators`.
    use shared::misc::serde_ignored::SerdeIgnored as NoDecorators;

    use super::*;
    use crate::misc::test_utils;

    #[test]
    fn test_minimal_forward() {
        let content = ForwardContent::builder()
            .to("test_to".to_owned())
            .msg(json!("test_msg"))
            .build();

        let expected = json! ({
            "to": content.to,
            "msg": content.msg
        });

        test_utils::test_msg(content, NoDecorators, RoutingTypeV1_0::Forward, expected);
    }
}
