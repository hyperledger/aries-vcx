use crate::messages::a2a::message_type::MessageType;
use crate::messages::a2a::message_family::MessageFamilies;
use crate::messages::mime_type::MimeType;
use crate::error::{VcxError, VcxErrorKind, VcxResult};

pub mod credential;
pub mod credential_offer;
pub mod credential_proposal;
pub mod credential_request;
pub mod credential_ack;

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
        let data_value = match mime_type {
            MimeType::Plain => {
                CredentialValue {
                    name: name.to_string(),
                    value: value.to_string(),
                    _type: None,
                }
            }
        };
        self.attributes.push(data_value);
        self
    }

    pub fn to_string(&self) -> VcxResult<String> {
        serde_json::to_string(&self.attributes)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed serialize credential preview attributes\nError: {}", err)))
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

#[cfg(test)]
pub mod test {
    use crate::messages::ack;
    use crate::messages::error;
    use crate::messages::issuance::credential_offer::test_utils::_credential_offer;

    pub fn _ack() -> ack::Ack {
        ack::test_utils::_ack().set_thread_id(&_credential_offer().id.0)
    }

    pub fn _problem_report() -> error::ProblemReport {
        error::tests::_problem_report().set_thread_id(&_credential_offer().id.0)
    }
}
