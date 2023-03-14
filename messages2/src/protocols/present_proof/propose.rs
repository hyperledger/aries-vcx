use std::str::FromStr;

use messages_macros::MessageContent;
use serde::{de::Error, Deserialize, Serialize};

use crate::{
    decorators::{Thread, Timing},
    misc::mime_type::MimeType,
    msg_types::{
        types::{
            present_proof::{PresentProof, PresentProofV1, PresentProofV1_0Kind},
            traits::MessageKind,
        },
        MessageType, Protocol,
    },
    protocols::traits::ConcreteMessage, Message,
};

pub type ProposePresentation = Message<ProposePresentationContent, ProposePresentationDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent)]
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

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
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

impl PresentationPreview {
    pub fn new(attributes: Vec<Attribute>, predicates: Vec<Predicate>) -> Self {
        Self {
            msg_type: PresentationPreviewMsgType,
            attributes,
            predicates,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct PresentationPreviewMsgType;

impl Serialize for PresentationPreviewMsgType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let protocol = Protocol::from(PresentProofV1_0Kind::parent());
        format_args!("{protocol}/{}", PresentProofV1_0Kind::PresentationPreview.as_ref()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PresentationPreviewMsgType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let msg_type = MessageType::deserialize(deserializer)?;

        if let Protocol::PresentProof(PresentProof::V1(PresentProofV1::V1_0(_))) = msg_type.protocol {
            if let Ok(PresentProofV1_0Kind::PresentationPreview) = PresentProofV1_0Kind::from_str(msg_type.kind) {
                return Ok(PresentationPreviewMsgType);
            }
        }

        let kind = PresentProofV1_0Kind::PresentationPreview;
        Err(D::Error::custom(format!("message kind is not {}", kind.as_ref())))
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
