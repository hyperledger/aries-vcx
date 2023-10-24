use std::str::FromStr;

use serde::{Deserialize, Serialize};
use shared_vcx::misc::utils::CowStr;
use typed_builder::TypedBuilder;

use crate::{
    decorators::{thread::Thread, timing::Timing},
    misc::MimeType,
    msg_parts::MsgParts,
    msg_types::{
        protocols::present_proof::{PresentProofType, PresentProofTypeV1, PresentProofTypeV1_0},
        traits::MessageKind,
        MessageType, Protocol,
    },
};

pub type ProposePresentationV1 =
    MsgParts<ProposePresentationV1Content, ProposePresentationV1Decorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct ProposePresentationV1Content {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub presentation_proposal: PresentationPreview,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct ProposePresentationV1Decorators {
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~thread")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread: Option<Thread>,
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PresentationPreview {
    #[serde(rename = "@type")]
    msg_type: PresentationPreviewMsgType,
    pub attributes: Vec<PresentationAttr>,
    pub predicates: Vec<Predicate>,
}

impl PresentationPreview {
    pub fn new(attributes: Vec<PresentationAttr>, predicates: Vec<Predicate>) -> Self {
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
#[derive(Copy, Clone, Debug, Default, Deserialize, PartialEq)]
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

        if let Protocol::PresentProofType(PresentProofType::V1(PresentProofTypeV1::V1_0(_))) =
            value.protocol
        {
            if let Ok(PresentProofTypeV1_0::PresentationPreview) =
                PresentProofTypeV1_0::from_str(value.kind)
            {
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, TypedBuilder)]
pub struct PresentationAttr {
    pub name: String,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cred_def_id: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mime-type")]
    pub mime_type: Option<MimeType>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referent: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, TypedBuilder)]
pub struct Predicate {
    pub name: String,
    pub predicate: PredicateOperator,
    pub threshold: i64,
    #[builder(default, setter(strip_option))]
    #[serde(flatten)]
    pub referent: Option<Referent>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Referent {
    pub cred_def_id: String,
    pub referent: String,
}

impl Referent {
    pub fn new(cred_def_id: String, referent: String) -> Self {
        Self {
            cred_def_id,
            referent,
        }
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
        let attribute = PresentationAttr::builder()
            .name("test_attribute_name".to_owned())
            .build();
        let predicate = Predicate::builder()
            .name("test_predicate_name".to_owned())
            .predicate(PredicateOperator::GreaterOrEqual)
            .threshold(1000)
            .build();
        let preview = PresentationPreview::new(vec![attribute], vec![predicate]);
        let content = ProposePresentationV1Content::builder()
            .presentation_proposal(preview)
            .build();

        let decorators = ProposePresentationV1Decorators::default();

        let expected = json!({
            "presentation_proposal": content.presentation_proposal
        });

        test_utils::test_msg(
            content,
            decorators,
            PresentProofTypeV1_0::ProposePresentation,
            expected,
        );
    }

    #[test]
    fn test_extended_propose_proof() {
        let attribute = PresentationAttr::builder()
            .name("test_attribute_name".to_owned())
            .build();
        let predicate = Predicate::builder()
            .name("test_predicate_name".to_owned())
            .predicate(PredicateOperator::GreaterOrEqual)
            .threshold(1000)
            .build();
        let preview = PresentationPreview::new(vec![attribute], vec![predicate]);
        let content = ProposePresentationV1Content::builder()
            .presentation_proposal(preview)
            .comment("test_comment".to_owned())
            .build();

        let decorators = ProposePresentationV1Decorators::builder()
            .thread(make_extended_thread())
            .timing(make_extended_timing())
            .build();

        let expected = json!({
            "comment": content.comment,
            "presentation_proposal": content.presentation_proposal,
            "~thread": decorators.thread,
            "~timing": decorators.timing
        });

        test_utils::test_msg(
            content,
            decorators,
            PresentProofTypeV1_0::ProposePresentation,
            expected,
        );
    }
}
