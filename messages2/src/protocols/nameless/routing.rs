//! Module containing the `mediator and relays` messages, as defined in the [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/concepts/0046-mediators-and-relays/README.md).

use serde::{Deserialize, Serialize};

use crate::{misc::utils::into_msg_with_type, msg_parts::MsgParts, msg_types::types::routing::RoutingProtocolV1_0};

pub type Forward = MsgParts<ForwardContent>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ForwardContent {
    pub to: String,
    pub msg: String,
}

impl ForwardContent {
    pub fn new(to: String, msg: String) -> Self {
        Self { to, msg }
    }
}

into_msg_with_type!(Forward, RoutingProtocolV1_0, Forward);

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::{test_utils, NoDecorators};

    #[test]
    fn test_minimal_forward() {
        let content = ForwardContent::new("test_to".to_owned(), "test_msg".to_owned());

        let expected = json! ({
            "to": content.to,
            "msg": content.msg
        });

        test_utils::test_msg(content, NoDecorators, RoutingProtocolV1_0::Forward, expected);
    }
}
