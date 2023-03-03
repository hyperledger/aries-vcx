mod ack;
mod issue_credential;
mod offer_credential;
mod propose_credential;
mod request_credential;

use derive_more::From;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use transitive::{TransitiveFrom, TransitiveTryFrom};

use crate::{
    composite_message::Message,
    delayed_serde::DelayedSerde,
    message_type::{
        message_family::cred_issuance::{
            CredentialIssuance as CredentialIssuanceKind, CredentialIssuanceV1, CredentialIssuanceV1_0,
        },
        MessageFamily, MessageType,
    },
    mime_type::MimeType,
    utils,
};

use self::{
    ack::AckCredential,
    issue_credential::{IssueCredential, IssueCredentialDecorators},
    offer_credential::{OfferCredential, OfferCredentialDecorators},
    propose_credential::{ProposeCredential, ProposeCredentialDecorators},
    request_credential::{RequestCredential, RequestCredentialDecorators},
};

use super::notification::AckDecorators;

#[derive(Clone, Debug, From)]
pub enum CredentialIssuance {
    OfferCredential(Message<OfferCredential, OfferCredentialDecorators>),
    ProposeCredential(Message<ProposeCredential, ProposeCredentialDecorators>),
    RequestCredential(Message<RequestCredential, RequestCredentialDecorators>),
    IssueCredential(Message<IssueCredential, IssueCredentialDecorators>),
    Ack(Message<AckCredential, AckDecorators>),
}

impl DelayedSerde for CredentialIssuance {
    type MsgType = CredentialIssuanceKind;

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let CredentialIssuanceKind::V1(major) = msg_type;
        let CredentialIssuanceV1::V1_0(minor) = major;

        match minor {
            CredentialIssuanceV1_0::OfferCredential => {
                Message::<OfferCredential, OfferCredentialDecorators>::delayed_deserialize(minor, deserializer)
                    .map(From::from)
            }
            CredentialIssuanceV1_0::ProposeCredential => {
                Message::<ProposeCredential, ProposeCredentialDecorators>::delayed_deserialize(minor, deserializer)
                    .map(From::from)
            }
            CredentialIssuanceV1_0::RequestCredential => {
                Message::<RequestCredential, RequestCredentialDecorators>::delayed_deserialize(minor, deserializer)
                    .map(From::from)
            }
            CredentialIssuanceV1_0::IssueCredential => {
                Message::<IssueCredential, IssueCredentialDecorators>::delayed_deserialize(minor, deserializer)
                    .map(From::from)
            }
            CredentialIssuanceV1_0::Ack => {
                Message::<AckCredential, AckDecorators>::delayed_deserialize(minor, deserializer).map(From::from)
            }
            CredentialIssuanceV1_0::CredentialPreview => Err(utils::not_standalone_msg::<D>(minor.as_ref())),
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::OfferCredential(v) => v.delayed_serialize(serializer),
            Self::ProposeCredential(v) => v.delayed_serialize(serializer),
            Self::RequestCredential(v) => v.delayed_serialize(serializer),
            Self::IssueCredential(v) => v.delayed_serialize(serializer),
            Self::Ack(v) => v.delayed_serialize(serializer),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CredentialPreview {
    #[serde(rename = "@type")]
    msg_type: CredentialPreviewMsgType,
    pub attributes: Vec<CredentialAttr>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, TransitiveFrom, TransitiveTryFrom)]
#[serde(into = "MessageType", try_from = "MessageType")]
#[transitive(try_from(MessageFamily, CredentialIssuanceKind, CredentialIssuanceV1, CredentialIssuanceV1_0))]
#[transitive(into(CredentialIssuanceV1_0, MessageType))]
struct CredentialPreviewMsgType;

impl From<CredentialPreviewMsgType> for CredentialIssuanceV1_0 {
    fn from(_value: CredentialPreviewMsgType) -> Self {
        CredentialIssuanceV1_0::CredentialPreview
    }
}

impl TryFrom<CredentialIssuanceV1_0> for CredentialPreviewMsgType {
    type Error = &'static str;

    fn try_from(value: CredentialIssuanceV1_0) -> Result<Self, Self::Error> {
        match value {
            CredentialIssuanceV1_0::CredentialPreview => Ok(Self),
            _ => Err("message kind is not \"credential_preview\""),
        }
    }
}

impl TryFrom<MessageType> for CredentialPreviewMsgType {
    type Error = &'static str;

    fn try_from(value: MessageType) -> Result<Self, Self::Error> {
        let interm = MessageFamily::from(value);
        CredentialPreviewMsgType::try_from(interm)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CredentialAttr {
    pub name: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<MimeType>,
}
