use crate::{
    a2a::{message_family::MessageFamilies, message_type::MessageType},
    concepts::mime_type::MimeType,
    errors::error::{MessagesError, MessagesErrorKind, MessagesResult},
};

pub mod credential;
pub mod credential_ack;
pub mod credential_offer;
pub mod credential_proposal;
pub mod credential_request;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CredentialPreviewData {
    #[serde(rename = "@type")]
    pub _type: MessageType,
    pub attributes: Vec<CredentialValue>,
}

impl CredentialPreviewData {
    pub fn new() -> Self {
        CredentialPreviewData::default()
    }

    pub fn add_value(mut self, name: &str, value: &str, mime_type: MimeType) -> CredentialPreviewData {
        let data_value = CredentialValue {
            name: name.to_string(),
            value: value.to_string(),
            _type: Some(mime_type),
        };
        self.attributes.push(data_value);
        self
    }

    pub fn to_string(&self) -> MessagesResult<String> {
        serde_json::to_string(&self.attributes).map_err(|err| {
            MessagesError::from_msg(
                MessagesErrorKind::SerializationError,
                format!("Failed serialize credential preview attributes\nError: {}", err),
            )
        })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct CredentialValue {
    pub name: String,
    pub value: String,
    #[serde(rename = "mime-type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _type: Option<MimeType>,
}

impl Default for CredentialPreviewData {
    fn default() -> CredentialPreviewData {
        CredentialPreviewData {
            _type: MessageType::build(MessageFamilies::CredentialIssuance, "credential-preview"),
            attributes: vec![],
        }
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use crate::{
        concepts::{ack, problem_report},
        protocols::issuance::credential_offer::test_utils::_credential_offer,
    };

    pub fn _credential_ack() -> ack::Ack {
        ack::test_utils::_ack().set_thread_id(&_credential_offer().id.0)
    }

    pub fn _problem_report() -> problem_report::ProblemReport {
        problem_report::test_utils::_problem_report().set_thread_id(&_credential_offer().id.0)
    }
}
