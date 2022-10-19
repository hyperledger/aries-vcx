use crate::a2a::{A2AMessage, MessageId};
use crate::ack::PleaseAck;
use crate::timing::Timing;
use crate::timing_optional;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct RevocationNotification {
    #[serde(rename = "@id")]
    id: MessageId,
    thread_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    please_ack: Option<PleaseAck>, // TODO: ["RECEIPT", "OUTCOME"]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    timing: Option<Timing>,
}

please_ack!(RevocationNotification);
a2a_message!(RevocationNotification);
timing_optional!(RevocationNotification);

impl RevocationNotification {
    pub fn create() -> Self {
        Self::default()
    }

    pub fn get_id(&self) -> MessageId {
        self.id.clone()
    }

    pub fn get_thread_id(&self) -> String {
        self.thread_id.clone()
    }

    pub fn set_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    pub fn set_thread_id(mut self, thread_id: String) -> Self {
        self.thread_id = thread_id;
        self
    }
}
