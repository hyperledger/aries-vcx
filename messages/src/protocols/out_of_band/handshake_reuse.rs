use crate::{
    a2a::{A2AMessage, MessageId},
    concepts::{thread::Thread, timing::Timing},
    timing_optional,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct OutOfBandHandshakeReuse {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

threadlike!(OutOfBandHandshakeReuse);
a2a_message!(OutOfBandHandshakeReuse);
timing_optional!(OutOfBandHandshakeReuse);
