use crate::{
    a2a::{message_family::MessageFamilies, message_type::MessageType, A2AMessage, MessageId},
    concepts::{mime_type::MimeType, thread::Thread, timing::Timing},
    errors::error::prelude::*,
    timing_optional,
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct PresentationProposal {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub presentation_proposal: PresentationPreview,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

timing_optional!(PresentationProposal);
threadlike_optional!(PresentationProposal);
a2a_message!(PresentationProposal);

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct PresentationPreview {
    #[serde(rename = "@type")]
    #[serde(default = "default_presentation_preview_type")]
    pub _type: MessageType,
    pub attributes: Vec<Attribute>,
    pub predicates: Vec<Predicate>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub cred_def_id: Option<String>,
    #[serde(rename = "mime-type")]
    pub mime_type: Option<MimeType>,
    pub value: Option<String>,
    pub referent: Option<String>,
}

impl Attribute {
    pub fn create(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Self::default()
        }
    }

    pub fn set_cred_def_id(mut self, cred_def_id: &str) -> Self {
        self.cred_def_id = Some(cred_def_id.to_string());
        self
    }

    pub fn set_value(mut self, value: &str) -> Self {
        self.value = Some(value.to_string());
        self
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq)]
pub struct Predicate {
    pub name: String,
    pub cred_def_id: Option<String>,
    pub predicate: String,
    pub threshold: i64,
    pub referent: Option<String>,
}

impl Predicate {
    pub fn create(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Self::default()
        }
    }

    pub fn set_cred_def_id(mut self, cred_def_id: &str) -> Self {
        self.cred_def_id = Some(cred_def_id.to_string());
        self
    }
}

fn default_presentation_preview_type() -> MessageType {
    MessageType::build(MessageFamilies::PresentProof, "presentation-preview")
}

impl PresentationProposal {
    pub fn create() -> Self {
        PresentationProposal::default()
    }

    pub fn set_id(mut self, id: &str) -> Self {
        self.id = MessageId(id.to_string());
        self
    }

    pub fn set_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn set_presentation_preview(mut self, presentation_preview: PresentationPreview) -> PresentationProposal {
        self.presentation_proposal = presentation_preview;
        self
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct PresentationProposalData {
    pub attributes: Vec<Attribute>,
    pub predicates: Vec<Predicate>,
    pub comment: Option<String>,
}

impl PresentationProposalData {
    pub fn create() -> Self {
        Self::default()
    }

    pub fn add_attribute(mut self, attr: Attribute) -> Self {
        self.attributes.push(attr);
        self
    }

    pub fn add_attribute_string(mut self, attr: &str) -> MessagesResult<Self> {
        let attr: Attribute = serde_json::from_str(attr).map_err(|err| {
            MessagesError::from_msg(
                MessagesErrorKind::InvalidJson,
                format!("Cannot deserialize supplied attribute: {:?}", err),
            )
        })?;
        self.attributes.push(attr);
        Ok(self)
    }

    pub fn set_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn add_predicate(mut self, pred: Predicate) -> Self {
        self.predicates.push(pred);
        self
    }

    pub fn add_predicate_string(mut self, pred: &str) -> MessagesResult<Self> {
        let pred: Predicate = serde_json::from_str(pred).map_err(|err| {
            MessagesError::from_msg(
                MessagesErrorKind::InvalidJson,
                format!("Cannot deserialize supplied predicate: {:?}", err),
            )
        })?;
        self.predicates.push(pred);
        Ok(self)
    }
}

impl From<PresentationProposalData> for PresentationProposal {
    fn from(data: PresentationProposalData) -> Self {
        Self {
            comment: data.comment,
            presentation_proposal: PresentationPreview {
                attributes: data.attributes,
                predicates: data.predicates,
                _type: default_presentation_preview_type(),
            },
            ..Self::default()
        }
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use super::*;
    use crate::protocols::connection::response::test_utils::_thread;

    fn _attachment() -> ::serde_json::Value {
        json!({"presentation": {}})
    }

    pub fn _comment() -> String {
        String::from("comment")
    }

    pub fn _presentation_proposal_data() -> PresentationProposalData {
        PresentationProposalData {
            attributes: vec![Attribute {
                name: String::from("name"),
                cred_def_id: None,
                mime_type: None,
                value: None,
                referent: None,
            }],
            predicates: vec![],
            comment: Some(String::from("comment")),
        }
    }

    pub fn _presentation_preview() -> PresentationPreview {
        PresentationPreview {
            attributes: vec![Attribute {
                name: String::from("name"),
                cred_def_id: None,
                mime_type: None,
                value: None,
                referent: None,
            }],
            predicates: vec![],
            ..Default::default()
        }
    }

    pub fn _presentation_proposal() -> PresentationProposal {
        PresentationProposal {
            id: MessageId::id(),
            comment: Some(_comment()),
            thread: Some(_thread()),
            presentation_proposal: _presentation_preview(),
            timing: None,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;
    use crate::protocols::proof_presentation::{
        presentation_proposal::test_utils::*, presentation_request::test_utils::thread_id,
    };

    #[test]
    fn test_presentation_proposal_build_works() {
        let presentation_proposal: PresentationProposal = PresentationProposal::default()
            .set_comment(_comment())
            .set_thread_id(&thread_id())
            .set_presentation_preview(_presentation_preview());

        assert_eq!(_presentation_proposal(), presentation_proposal);
    }
}
