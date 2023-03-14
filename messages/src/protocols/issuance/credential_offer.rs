use crate::{
    a2a::{A2AMessage, MessageId},
    concepts::{
        attachment::{AttachmentId, Attachments},
        mime_type::MimeType,
        thread::Thread,
        timing::Timing,
    },
    errors::error::MessagesResult,
    protocols::issuance::CredentialPreviewData,
    timing_optional,
};

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
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

threadlike_optional!(CredentialOffer);
a2a_message!(CredentialOffer);
timing_optional!(CredentialOffer);

impl CredentialOffer {
    pub fn create() -> Self {
        CredentialOffer::default()
    }

    pub fn set_id(mut self, id: &str) -> Self {
        self.id = MessageId(id.to_string());
        self
    }

    pub fn set_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    pub fn set_offers_attach(mut self, credential_offer: &str) -> MessagesResult<CredentialOffer> {
        self.offers_attach.add_base64_encoded_json_attachment(
            AttachmentId::CredentialOffer,
            ::serde_json::Value::String(credential_offer.to_string()),
        )?;
        Ok(self)
    }

    pub fn set_credential_preview_data(mut self, credential_preview: CredentialPreviewData) -> CredentialOffer {
        self.credential_preview = credential_preview;
        self
    }

    pub fn add_credential_preview_data(mut self, name: &str, value: &str, mime_type: MimeType) -> CredentialOffer {
        self.credential_preview = self.credential_preview.add_value(name, value, mime_type);
        self
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OfferInfo {
    pub credential_json: String,
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
}

impl OfferInfo {
    pub fn new(
        credential_json: String,
        cred_def_id: String,
        rev_reg_id: Option<String>,
        tails_file: Option<String>,
    ) -> Self {
        Self {
            credential_json,
            cred_def_id,
            rev_reg_id,
            tails_file,
        }
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use super::*;
    use crate::protocols::connection::response::test_utils::_thread;

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
        CredentialPreviewData::new().add_value(name, value, MimeType::Plain)
    }

    pub fn thread() -> Thread {
        Thread::new().set_thid(_credential_offer().id.0)
    }

    pub fn thread_1() -> Thread {
        Thread::new().set_thid("testid_1".into())
    }

    pub fn thread_id() -> String {
        thread().thid.unwrap()
    }

    pub fn _cred_def_id() -> String {
        String::from("cred_def_id:id")
    }

    pub fn _rev_reg_id() -> String {
        String::from("TEST_REV_REG_ID")
    }

    pub fn _tails_file() -> String {
        String::from("TEST_TAILS_FILE")
    }

    pub fn _credential_offer() -> CredentialOffer {
        let mut attachment = Attachments::new();
        attachment
            .add_base64_encoded_json_attachment(AttachmentId::CredentialOffer, _attachment())
            .unwrap();

        CredentialOffer {
            id: MessageId::id(),
            comment: _comment(),
            credential_preview: _preview_data(),
            offers_attach: attachment,
            thread: Some(_thread()),
            timing: None,
        }
    }

    pub fn _offer_info() -> OfferInfo {
        OfferInfo {
            credential_json: _preview_data().to_string().unwrap(),
            cred_def_id: _cred_def_id(),
            rev_reg_id: Some(_rev_reg_id()),
            tails_file: Some(_tails_file()),
        }
    }

    pub fn _offer_info_unrevokable() -> OfferInfo {
        OfferInfo {
            credential_json: _preview_data().to_string().unwrap(),
            cred_def_id: _cred_def_id(),
            rev_reg_id: None,
            tails_file: None,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;
    use crate::protocols::{connection::response::test_utils::_thread_id, issuance::credential_offer::test_utils::*};

    #[test]
    fn test_credential_offer_build_works() {
        let credential_offer: CredentialOffer = CredentialOffer::create()
            .set_comment(_comment())
            .set_thread_id(&_thread_id())
            .set_credential_preview_data(_preview_data())
            .set_offers_attach(&_attachment().to_string())
            .unwrap();

        assert_eq!(_credential_offer(), credential_offer);
    }
}
