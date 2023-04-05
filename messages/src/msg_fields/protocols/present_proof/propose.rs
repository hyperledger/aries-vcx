use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    decorators::{thread::Thread, timing::Timing},
    misc::{utils::CowStr, MimeType},
    msg_parts::MsgParts,
    msg_types::{
        protocols::present_proof::{PresentProofType, PresentProofTypeV1, PresentProofTypeV1_0},
        traits::MessageKind,
        MessageType, Protocol,
    },
};

pub type ProposePresentation = MsgParts<ProposePresentationContent, ProposePresentationDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
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

/// Non-standalone message type.
/// This is only encountered as part of an existent message.
/// It is not a message on it's own.
#[derive(Copy, Clone, Debug, Deserialize, PartialEq)]
#[serde(try_from = "CowStr")]
struct PresentationPreviewMsgType;

impl<'a> From<&'a PresentationPreviewMsgType> for PresentProofTypeV1_0 {
    fn from(_value: &'a PresentationPreviewMsgType) -> Self {
        PresentProofTypeV1_0::PresentationPreview
    }
}

impl<'a> TryFrom<CowStr<'a>> for PresentationPreviewMsgType {
    type Error = String;

    fn try_from(value: CowStr<'a>) -> Result<Self, Self::Error> {
        let value = MessageType::try_from(value.0.as_ref())?;

        if let Protocol::PresentProofType(PresentProofType::V1(PresentProofTypeV1::V1_0(_))) = value.protocol {
            if let Ok(PresentProofTypeV1_0::PresentationPreview) = PresentProofTypeV1_0::from_str(value.kind) {
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
        let protocol = Protocol::from(PresentProofTypeV1_0::parent());
        let kind = PresentProofTypeV1_0::from(self);
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

        let expected = json!({
            "presentation_proposal": content.presentation_proposal
        });

        test_utils::test_msg(content, decorators, PresentProofTypeV1_0::ProposePresentation, expected);
    }

    #[test]
    fn test_extended_propose_proof() {
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

        let expected = json!({
            "comment": content.comment,
            "presentation_proposal": content.presentation_proposal,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(content, decorators, PresentProofTypeV1_0::ProposePresentation, expected);
    }
}
