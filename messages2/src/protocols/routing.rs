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
    use crate::{AriesMessage, Message};

    const FORWARD: &str = "https://didcomm.org/routing/1.0/forward";

    #[test]
    fn test_minimal_message() {
        let to = "test".to_owned();
        let id = "test".to_owned();
        let msg_value = to.clone();

        let content = ForwardContent::new(to.clone(), msg_value.clone());
        let msg = Message::new(id.clone(), content);
        let msg = AriesMessage::from(msg);

        let json = json! ({
            "@type": FORWARD,
            "@id": id,
            "to": to,
            "msg": msg_value
        });

        let deserialized = AriesMessage::deserialize(&json).unwrap();

        assert_eq!(serde_json::to_value(&msg).unwrap(), json);
        assert_eq!(deserialized, msg);
    }

    #[test]
    #[should_panic]
    fn test_incomplete_message() {
        let to = "test".to_owned();

        let json = json!({
            "@type": FORWARD,
            "@id": "test",
            "to": to
        });

        AriesMessage::deserialize(&json).unwrap();
    }
}
