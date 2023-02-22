mod ack;
mod issue_credential;
mod offer_credential;
mod propose_credential;
mod request_credential;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use transitive::{TransitiveFrom, TransitiveTryFrom};

use crate::{
    delayed_serde::DelayedSerde,
    message_type::{
        message_family::cred_issuance::{
            CredentialIssuance as CredentialIssuanceKind, CredentialIssuanceV1, CredentialIssuanceV1_0,
        },
        MessageFamily, MessageType,
    },
    mime_type::MimeType,
};

use self::{
    ack::AckCredential, issue_credential::IssueCredential, offer_credential::OfferCredential,
    propose_credential::ProposeCredential, request_credential::RequestCredential,
};

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

    fn delayed_deserialize<'de, D>(seg: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let CredentialIssuanceKind::V1(major) = seg;
        let CredentialIssuanceV1::V1_0(minor) = major;

        match minor {
            CredentialIssuanceV1_0::OfferCredential => OfferCredential::deserialize(deserializer).map(From::from),
            CredentialIssuanceV1_0::ProposeCredential => ProposeCredential::deserialize(deserializer).map(From::from),
            CredentialIssuanceV1_0::RequestCredential => RequestCredential::deserialize(deserializer).map(From::from),
            CredentialIssuanceV1_0::IssueCredential => IssueCredential::deserialize(deserializer).map(From::from),
            CredentialIssuanceV1_0::Ack => AckCredential::deserialize(deserializer).map(From::from),
            CredentialIssuanceV1_0::CredentialPreview => Err(D::Error::custom(concat!(
                stringify!(CredentialIssuanceV1_0::CredentialPreview),
                " is not a standalone message"
            ))),
        }
    }

    fn delayed_serialize<'a, M, F, S>(&self, state: &'a mut M, closure: &mut F) -> Result<S::Ok, S::Error>
    where
        M: serde::ser::SerializeMap,
        F: FnMut(&'a mut M) -> S,
        S: serde::Serializer,
        S::Error: From<M::Error>,
    {
        match self {
            Self::OfferCredential(v) => v.delayed_serialize(state, closure),
            Self::ProposeCredential(v) => v.delayed_serialize(state, closure),
            Self::RequestCredential(v) => v.delayed_serialize(state, closure),
            Self::IssueCredential(v) => v.delayed_serialize(state, closure),
            Self::Ack(v) => v.delayed_serialize(state, closure),
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
#[transitive(into(all(CredentialIssuanceV1_0, CredentialIssuanceV1, MessageFamily, MessageType)))]
#[transitive(try_from(MessageFamily, CredentialIssuanceKind, CredentialIssuanceV1, CredentialIssuanceV1_0))]
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
