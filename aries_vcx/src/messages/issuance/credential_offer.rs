use crate::error::VcxResult;
use crate::messages::a2a::{A2AMessage, MessageId};
use crate::messages::attachment::{AttachmentId, Attachments};
use crate::messages::issuance::CredentialPreviewData;
use crate::messages::mime_type::MimeType;
use crate::messages::thread::Thread;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct CredentialOffer {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub credential_preview: CredentialPreviewData,
    #[serde(rename = "offers~attach")]
    pub offers_attach: Attachments,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
}

impl CredentialOffer {
    pub fn create() -> Self {
        CredentialOffer::default()
    }

    pub fn set_id(mut self, id: String) -> Self {
        self.id = MessageId(id);
        self
    }

    pub fn set_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    pub fn set_offers_attach(mut self, credential_offer: &str) -> VcxResult<CredentialOffer> {
        self.offers_attach.add_base64_encoded_json_attachment(AttachmentId::CredentialOffer, ::serde_json::Value::String(credential_offer.to_string()))?;
        Ok(self)
    }

    pub fn set_credential_preview_data(mut self, credential_preview: CredentialPreviewData) -> VcxResult<CredentialOffer> {
        self.credential_preview = credential_preview;
        Ok(self)
    }

    pub fn add_credential_preview_data(mut self, name: &str, value: &str, mime_type: MimeType) -> VcxResult<CredentialOffer> {
        self.credential_preview = self.credential_preview.add_value(name, value, mime_type)?;
        Ok(self)
    }

    pub fn set_thread_id(mut self, id: &str) -> Self {
        self.thread = Some(Thread::new().set_thid(id.to_string()));
        self
    }
}

a2a_message!(CredentialOffer);

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use crate::messages::connection::response::test_utils::_thread;

    use super::*;

    pub fn _attachment() -> ::serde_json::Value {
        json!({
            "schema_id":"NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0",
            "cred_def_id":"NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG1"
        })
    }

    pub fn _comment() -> Option<String> {
        Some(String::from("comment"))
    }

    pub fn _value() -> (&'static str, &'static str) {
        ("attribute", "value")
    }

    pub fn _preview_data() -> CredentialPreviewData {
        let (name, value) = _value();
        CredentialPreviewData::new()
            .add_value(name, value, MimeType::Plain).unwrap()
    }

    pub fn thread() -> Thread {
        Thread::new().set_thid(_credential_offer().id.0)
    }

    pub fn thread_id() -> String {
        thread().thid.unwrap()
    }

    pub fn _credential_offer() -> CredentialOffer {
        let mut attachment = Attachments::new();
        attachment.add_base64_encoded_json_attachment(AttachmentId::CredentialOffer, _attachment()).unwrap();

        CredentialOffer {
            id: MessageId::id(),
            comment: _comment(),
            credential_preview: _preview_data(),
            offers_attach: attachment,
            thread: Some(_thread()),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::messages::connection::response::test_utils::_thread_id;
    use crate::messages::issuance::credential_offer::test_utils::*;

    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_credential_offer_build_works() {
        let credential_offer: CredentialOffer = CredentialOffer::create()
            .set_comment(_comment())
            .set_thread_id(&_thread_id())
            .set_credential_preview_data(_preview_data()).unwrap()
            .set_offers_attach(&_attachment().to_string()).unwrap();

        assert_eq!(_credential_offer(), credential_offer);
    }
}
