use std::str::FromStr;

use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};

use crate::{
    decorators::{thread::Thread, timing::Timing},
    misc::mime_type::MimeType,
    msg_types::{
        types::{
            present_proof::{PresentProof, PresentProofV1, PresentProofV1_0Kind},
            traits::MessageKind,
        },
        MessageType, Protocol,
    },
    Message,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Predicate {
    pub name: String,
    pub predicate: PredicateOperator,
    pub threshold: i64,
    #[serde(flatten)]
    pub referent: Option<Referent>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Referent {
    pub cred_def_id: String,
    pub referent: String,
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
