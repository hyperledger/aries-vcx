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
    use crate::misc::{nothing::Nothing, test_utils};

    #[test]
    fn test_minimal_forward() {
        let content = ForwardContent::new("test_to".to_owned(), "test_msg".to_owned());

        let json = json! ({
            "to": content.to,
            "msg": content.msg
        });

        test_utils::test_msg::<ForwardContent, _, _>(content, Nothing, json);
    }
}
