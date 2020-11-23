use crate::aries::messages::a2a::{A2AMessage, MessageId};
use crate::aries::messages::ack::PleaseAck;
use crate::aries::messages::attachment::{AttachmentId, Attachments};
use crate::aries::messages::thread::Thread;
use crate::error::prelude::*;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Presentation {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "presentations~attach")]
    pub presentations_attach: Attachments,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~please_ack")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub please_ack: Option<PleaseAck>,
}

impl Presentation {
    pub fn create() -> Self {
        Presentation::default()
    }

    pub fn set_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn set_presentations_attach(mut self, presentations: String) -> VcxResult<Presentation> {
        self.presentations_attach.add_base64_encoded_json_attachment(AttachmentId::Presentation, serde_json::Value::String(presentations))?;
        Ok(self)
    }
}

please_ack!(Presentation);
threadlike!(Presentation);
a2a_message!(Presentation);


#[cfg(test)]
pub mod tests {
    use crate::aries::messages::proof_presentation::presentation_request::tests::{thread, thread_id};

    use super::*;

    fn _attachment() -> serde_json::Value {
        json!({"presentation": {}})
    }

    fn _comment() -> String {
        String::from("comment")
    }

    pub fn _presentation() -> Presentation {
        let mut attachment = Attachments::new();
        attachment.add_base64_encoded_json_attachment(AttachmentId::Presentation, _attachment()).unwrap();

        Presentation {
            id: MessageId::id(),
            comment: Some(_comment()),
            presentations_attach: attachment,
            thread: thread(),
            please_ack: Some(PleaseAck {}),
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_presentation_build_works() {
        let presentation: Presentation = Presentation::default()
            .set_comment(_comment())
            .ask_for_ack()
            .set_thread_id(&thread_id())
            .set_presentations_attach(_attachment().to_string()).unwrap();

        assert_eq!(_presentation(), presentation);
    }
}
