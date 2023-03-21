use std::str::FromStr;

use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{thread::Thread, timing::Timing},
    message::Message,
    misc::mime_type::MimeType,
    msg_types::{
        types::{
            present_proof::{PresentProof, PresentProofV1, PresentProofV1_0Kind},
            traits::MessageKind,
        },
        MessageType, Protocol,
    },
};

pub type ProposePresentation = Message<ProposePresentationContent, ProposePresentationDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, PartialEq)]
#[message(kind = "PresentProofV1_0Kind::ProposePresentation")]
pub struct ProposePresentationContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub presentation_proposal: PresentationPreview,
}

impl ProposePresentationContent {
    pub fn new(presentation_proposal: PresentationPreview) -> Self {
        Self {
            comment: None,
            presentation_proposal,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct ProposePresentationDecorators {
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PresentationPreview {
    #[serde(rename = "@type")]
    msg_type: PresentationPreviewMsgType,
    pub attributes: Vec<Attribute>,
    pub predicates: Vec<Predicate>,
}

impl PresentationPreview {
    pub fn new(attributes: Vec<Attribute>, predicates: Vec<Predicate>) -> Self {
        Self {
            msg_type: PresentationPreviewMsgType,
            attributes,
            predicates,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq)]
#[serde(try_from = "MessageType")]
struct PresentationPreviewMsgType;

impl<'a> From<&'a PresentationPreviewMsgType> for PresentProofV1_0Kind {
    fn from(_value: &'a PresentationPreviewMsgType) -> Self {
        PresentProofV1_0Kind::PresentationPreview
    }
}

impl<'a> TryFrom<MessageType<'a>> for PresentationPreviewMsgType {
    type Error = String;

    fn try_from(value: MessageType<'a>) -> Result<Self, Self::Error> {
        if let Protocol::PresentProof(PresentProof::V1(PresentProofV1::V1_0(_))) = value.protocol {
            if let Ok(PresentProofV1_0Kind::PresentationPreview) = PresentProofV1_0Kind::from_str(value.kind) {
                return Ok(PresentationPreviewMsgType);
            }
        }
        Err(format!("message kind is not {}", value.kind))
    }
}

impl Serialize for PresentationPreviewMsgType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let protocol = Protocol::from(PresentProofV1_0Kind::parent());
        let kind = PresentProofV1_0Kind::from(self);
        format_args!("{protocol}/{}", kind.as_ref()).serialize(serializer)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub cred_def_id: Option<String>,
    #[serde(rename = "mime-type")]
    pub mime_type: Option<MimeType>,
    pub value: Option<String>,
    pub referent: Option<String>,
}

impl Attribute {
    pub fn new(name: String) -> Self {
        Self {
            name,
            cred_def_id: None,
            mime_type: None,
            value: None,
            referent: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Predicate {
    pub name: String,
    pub predicate: PredicateOperator,
    pub threshold: i64,
    #[serde(flatten)]
    pub referent: Option<Referent>,
}

impl Predicate {
    pub fn new(name: String, predicate: PredicateOperator, threshold: i64) -> Self {
        Self {
            name,
            predicate,
            threshold,
            referent: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Referent {
    pub cred_def_id: String,
    pub referent: String,
}

impl Referent {
    pub fn new(cred_def_id: String, referent: String) -> Self {
        Self { cred_def_id, referent }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum PredicateOperator {
    #[serde(rename = ">=")]
    GreaterOrEqual,
    #[serde(rename = "<=")]
    LessOrEqual,
    #[serde(rename = ">")]
    GreterThan,
    #[serde(rename = "<")]
    LessThan,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{thread::tests::make_extended_thread, timing::tests::make_extended_timing},
        misc::test_utils,
    };

    #[test]
    fn test_minimal_propose_proof() {
        let attribute = Attribute::new("test_attribute_name".to_owned());
        let predicate = Predicate::new(
            "test_predicate_name".to_owned(),
            PredicateOperator::GreaterOrEqual,
            1000,
        );
        let preview = PresentationPreview::new(vec![attribute], vec![predicate]);
        let content = ProposePresentationContent::new(preview);

        let decorators = ProposePresentationDecorators::default();

        let json = json!({
            "presentation_proposal": content.presentation_proposal
        });

        test_utils::test_msg::<ProposePresentationContent, _, _>(content, decorators, json);
    }

    #[test]
    fn test_extensive_propose_proof() {
        let attribute = Attribute::new("test_attribute_name".to_owned());
        let predicate = Predicate::new(
            "test_predicate_name".to_owned(),
            PredicateOperator::GreaterOrEqual,
            1000,
        );
        let preview = PresentationPreview::new(vec![attribute], vec![predicate]);
        let mut content = ProposePresentationContent::new(preview);
        content.comment = Some("test_comment".to_owned());

        let mut decorators = ProposePresentationDecorators::default();
        decorators.thread = Some(make_extended_thread());
        decorators.timing = Some(make_extended_timing());

        let json = json!({
            "comment": content.comment,
            "presentation_proposal": content.presentation_proposal,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg::<ProposePresentationContent, _, _>(content, decorators, json);
    }
}
