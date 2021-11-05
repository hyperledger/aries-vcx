use crate::error::prelude::*;
use crate::messages::a2a::{A2AMessage, MessageId};
use crate::messages::ack::PleaseAck;
use crate::messages::attachment::{AttachmentId, Attachments};
use crate::messages::thread::Thread;

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

    pub fn set_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
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

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use crate::messages::proof_presentation::presentation_request::test_utils::_thread;
    use crate::messages::connection::response::test_utils::_thread_1;

    use super::*;

    pub fn _attachment() -> serde_json::Value {
        json!({"presentation": {}})
    }

    pub fn _comment() -> Option<String> {
        Some(String::from("comment"))
    }

    pub fn _presentation() -> Presentation {
        let mut attachment = Attachments::new();
        attachment.add_base64_encoded_json_attachment(AttachmentId::Presentation, _attachment()).unwrap();

        Presentation {
            id: MessageId::id(),
            comment: _comment(),
            presentations_attach: attachment,
            thread: _thread(),
            please_ack: Some(PleaseAck {}),
        }
    }

    pub fn _presentation_1() -> Presentation {
        let mut attachment = Attachments::new();
        attachment.add_base64_encoded_json_attachment(AttachmentId::Presentation, _attachment()).unwrap();

        Presentation {
            id: MessageId::id(),
            comment: _comment(),
            presentations_attach: attachment,
            thread: _thread_1(),
            please_ack: Some(PleaseAck {}),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod tests {
    use crate::messages::proof_presentation::presentation::test_utils::*;
    use crate::messages::proof_presentation::presentation_request::test_utils::thread_id;

    use super::*;

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
