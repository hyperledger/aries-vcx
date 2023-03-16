use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{msg_types::types::routing::RoutingV1_0Kind, Message};

pub type Forward = Message<ForwardContent>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "RoutingV1_0Kind::Forward")]
pub struct ForwardContent {
    pub to: String,
    pub msg: String,
}

impl ForwardContent {
    pub fn new(to: String, msg: String) -> Self {
        Self { to, msg }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{misc::{test_utils, nothing::Nothing}};

    #[test]
    fn test_minimal_message() {
        let msg_type = test_utils::build_msg_type::<ForwardContent>();

        let to = "test".to_owned();
        let msg_value = to.clone();
        let content = ForwardContent::new(to.clone(), msg_value.clone());

        let json = json! ({
            "@type": msg_type,
            "to": to,
            "msg": msg_value
        });

        test_utils::test_msg(content, Nothing, json);
    }
}
