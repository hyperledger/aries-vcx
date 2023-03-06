use messages_macros::MessageContent;
use serde::{Deserialize, Serialize};
use transitive::{TransitiveFrom, TransitiveTryFrom};

use crate::{
    composite_message::Message,
    decorators::{Thread, Timing},
    message_type::{
        message_family::present_proof::{PresentProof, PresentProofV1, PresentProofV1_0},
        MessageFamily, MessageType,
    },
    mime_type::MimeType,
    protocols::traits::MessageKind,
};

pub type ProposePresentation = Message<ProposePresentationContent, ProposePresentationDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
#[message(kind = "PresentProofV1_0::ProposePresentation")]
pub struct ProposePresentationContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub presentation_proposal: PresentationPreview,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProposePresentationDecorators {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PresentationPreview {
    #[serde(rename = "@type")]
    msg_type: PresentationPreviewMsgType,
    pub attributes: Vec<Attribute>,
    pub predicates: Vec<Predicate>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, TransitiveFrom, TransitiveTryFrom)]
#[serde(into = "MessageType", try_from = "MessageType")]
#[transitive(try_from(MessageFamily, PresentProof, PresentProofV1, PresentProofV1_0))]
#[transitive(into(PresentProofV1_0, MessageType))]
struct PresentationPreviewMsgType;

impl From<PresentationPreviewMsgType> for PresentProofV1_0 {
    fn from(_value: PresentationPreviewMsgType) -> Self {
        PresentProofV1_0::PresentationPreview
    }
}

impl TryFrom<PresentProofV1_0> for PresentationPreviewMsgType {
    type Error = &'static str;

    fn try_from(value: PresentProofV1_0) -> Result<Self, Self::Error> {
        match value {
            PresentProofV1_0::PresentationPreview => Ok(Self),
            _ => Err("message kind is not \"presentation-preview\""),
        }
    }
}

impl TryFrom<MessageType> for PresentationPreviewMsgType {
    type Error = &'static str;

    fn try_from(value: MessageType) -> Result<Self, Self::Error> {
        let interm = MessageFamily::from(value);
        PresentationPreviewMsgType::try_from(interm)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Attribute {
    pub name: String,
    pub cred_def_id: Option<String>,
    #[serde(rename = "mime-type")]
    pub mime_type: Option<MimeType>,
    pub value: Option<String>,
    pub referent: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Predicate {
    pub name: String,
    pub predicate: PredicateOperator,
    pub threshold: i64,
    #[serde(flatten)]
    pub referent: Option<Referent>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Referent {
    pub cred_def_id: String,
    pub referent: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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
