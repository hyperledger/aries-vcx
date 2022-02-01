use crate::messages::a2a::{A2AMessage, MessageId};
use crate::messages::thread::Thread;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct OutOfBandHandshakeReuse {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "~thread")]
    pub thread: Thread,
}

threadlike!(OutOfBandHandshakeReuse);
a2a_message!(OutOfBandHandshakeReuse);
