mod ack;
mod issue_credential;
mod offer_credential;
mod propose_credential;
mod request_credential;

use derive_more::From;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use transitive::{TransitiveFrom, TransitiveTryFrom};

use crate::{
    composite_message::{transit_to_aries_msg, Message},
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
    ack::AckCredentialContent,
    issue_credential::{IssueCredentialContent, IssueCredentialDecorators},
    offer_credential::{OfferCredentialContent, OfferCredentialDecorators},
    propose_credential::{ProposeCredentialContent, ProposeCredentialDecorators},
    request_credential::{RequestCredentialContent, RequestCredentialDecorators},
};

pub use self::{
    ack::AckCredential, issue_credential::IssueCredential, offer_credential::OfferCredential,
    propose_credential::ProposeCredential, request_credential::RequestCredential,
};

use super::notification::AckDecorators;

#[derive(Clone, Debug, From)]
pub enum CredentialIssuance {
    OfferCredential(OfferCredential),
    ProposeCredential(ProposeCredential),
    RequestCredential(RequestCredential),
    IssueCredential(IssueCredential),
    Ack(AckCredential),
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
                OfferCredential::delayed_deserialize(minor, deserializer).map(From::from)
            }
            CredentialIssuanceV1_0::ProposeCredential => {
                ProposeCredential::delayed_deserialize(minor, deserializer).map(From::from)
            }
            CredentialIssuanceV1_0::RequestCredential => {
                RequestCredential::delayed_deserialize(minor, deserializer).map(From::from)
            }
            CredentialIssuanceV1_0::IssueCredential => {
                IssueCredential::delayed_deserialize(minor, deserializer).map(From::from)
            }
            CredentialIssuanceV1_0::Ack => AckCredential::delayed_deserialize(minor, deserializer).map(From::from),
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

impl CredentialPreview {
    pub fn new(attributes: Vec<CredentialAttr>) -> Self {
        Self {
            msg_type: CredentialPreviewMsgType,
            attributes,
        }
    }
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

transit_to_aries_msg!(OfferCredentialContent: OfferCredentialDecorators, CredentialIssuance);
transit_to_aries_msg!(
    ProposeCredentialContent: ProposeCredentialDecorators,
    CredentialIssuance
);
transit_to_aries_msg!(
    RequestCredentialContent: RequestCredentialDecorators,
    CredentialIssuance
);
transit_to_aries_msg!(IssueCredentialContent: IssueCredentialDecorators, CredentialIssuance);
transit_to_aries_msg!(AckCredentialContent: AckDecorators, CredentialIssuance);
