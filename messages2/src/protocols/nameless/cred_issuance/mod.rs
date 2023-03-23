//! Module containing the `issue credential` protocol messages, as defined in the [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/features/0036-issue-credential/README.md).

pub mod ack;
pub mod issue_credential;
pub mod offer_credential;
pub mod propose_credential;
pub mod request_credential;

use std::str::FromStr;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use self::{
    ack::{AckCredential, AckCredentialContent},
    issue_credential::{IssueCredential, IssueCredentialContent, IssueCredentialDecorators},
    offer_credential::{OfferCredential, OfferCredentialContent, OfferCredentialDecorators},
    propose_credential::{ProposeCredential, ProposeCredentialContent, ProposeCredentialDecorators},
    request_credential::{RequestCredential, RequestCredentialContent, RequestCredentialDecorators},
};
use super::notification::AckDecorators;
use crate::{
    misc::{
        utils::{self, transit_to_aries_msg},
        MimeType,
    },
    msg_types::{
        traits::MessageKind,
        types::cred_issuance::{
            CredentialIssuance as CredentialIssuanceKind, CredentialIssuanceV1, CredentialIssuanceV1_0,
        },
        MessageType, Protocol,
    },
    protocols::traits::DelayedSerde,
};

#[derive(Clone, Debug, From, PartialEq)]
pub enum CredentialIssuance {
    OfferCredential(OfferCredential),
    ProposeCredential(ProposeCredential),
    RequestCredential(RequestCredential),
    IssueCredential(IssueCredential),
    Ack(AckCredential),
}

impl DelayedSerde for CredentialIssuance {
    type MsgType<'a> = (CredentialIssuanceKind, &'a str);

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (major, kind) = msg_type;
        let CredentialIssuanceKind::V1(major) = major;
        let CredentialIssuanceV1::V1_0(_minor) = major;
        let kind = CredentialIssuanceV1_0::from_str(kind).map_err(D::Error::custom)?;

        match kind {
            CredentialIssuanceV1_0::OfferCredential => {
                OfferCredential::delayed_deserialize(kind, deserializer).map(From::from)
            }
            CredentialIssuanceV1_0::ProposeCredential => {
                ProposeCredential::delayed_deserialize(kind, deserializer).map(From::from)
            }
            CredentialIssuanceV1_0::RequestCredential => {
                RequestCredential::delayed_deserialize(kind, deserializer).map(From::from)
            }
            CredentialIssuanceV1_0::IssueCredential => {
                IssueCredential::delayed_deserialize(kind, deserializer).map(From::from)
            }
            CredentialIssuanceV1_0::Ack => AckCredential::delayed_deserialize(kind, deserializer).map(From::from),
            CredentialIssuanceV1_0::CredentialPreview => Err(utils::not_standalone_msg::<D>(kind.as_ref())),
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Copy, Clone, Debug, Deserialize, PartialEq)]
#[serde(try_from = "MessageType")]
struct CredentialPreviewMsgType;

impl<'a> From<&'a CredentialPreviewMsgType> for CredentialIssuanceV1_0 {
    fn from(_value: &'a CredentialPreviewMsgType) -> Self {
        CredentialIssuanceV1_0::CredentialPreview
    }
}

impl<'a> TryFrom<MessageType<'a>> for CredentialPreviewMsgType {
    type Error = String;

    fn try_from(value: MessageType) -> Result<Self, Self::Error> {
        if let Protocol::CredentialIssuance(CredentialIssuanceKind::V1(CredentialIssuanceV1::V1_0(_))) = value.protocol
        {
            if let Ok(CredentialIssuanceV1_0::CredentialPreview) = CredentialIssuanceV1_0::from_str(value.kind) {
                return Ok(CredentialPreviewMsgType);
            }
        }

        Err(format!("message kind is not {}", value.kind))
    }
}

impl Serialize for CredentialPreviewMsgType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let protocol = Protocol::from(CredentialIssuanceV1_0::parent());
        let kind = CredentialIssuanceV1_0::from(self);
        format_args!("{protocol}/{}", kind.as_ref()).serialize(serializer)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct CredentialAttr {
    pub name: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<MimeType>,
}

impl CredentialAttr {
    pub fn new(name: String, value: String) -> Self {
        Self {
            name,
            value,
            mime_type: None,
        }
    }
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
