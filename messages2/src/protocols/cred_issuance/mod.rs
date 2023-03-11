mod ack;
mod issue_credential;
mod offer_credential;
mod propose_credential;
mod request_credential;

use std::str::FromStr;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    composite_message::{transit_to_aries_msg, Message},
    delayed_serde::DelayedSerde,
    message_type::{
        message_protocol::{
            cred_issuance::{
                CredentialIssuance as CredentialIssuanceKind, CredentialIssuanceV1, CredentialIssuanceV1_0Kind,
            },
            traits::MessageKind,
        },
        serde::MessageType,
        MessageFamily,
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
    type MsgType<'a> = (CredentialIssuanceKind, &'a str);

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (major, kind) = msg_type;
        let CredentialIssuanceKind::V1(major) = major;
        let CredentialIssuanceV1::V1_0(_minor) = major;
        let kind = CredentialIssuanceV1_0Kind::from_str(kind).map_err(D::Error::custom)?;

        match kind {
            CredentialIssuanceV1_0Kind::OfferCredential => {
                OfferCredential::delayed_deserialize(kind, deserializer).map(From::from)
            }
            CredentialIssuanceV1_0Kind::ProposeCredential => {
                ProposeCredential::delayed_deserialize(kind, deserializer).map(From::from)
            }
            CredentialIssuanceV1_0Kind::RequestCredential => {
                RequestCredential::delayed_deserialize(kind, deserializer).map(From::from)
            }
            CredentialIssuanceV1_0Kind::IssueCredential => {
                IssueCredential::delayed_deserialize(kind, deserializer).map(From::from)
            }
            CredentialIssuanceV1_0Kind::Ack => AckCredential::delayed_deserialize(kind, deserializer).map(From::from),
            CredentialIssuanceV1_0Kind::CredentialPreview => Err(utils::not_standalone_msg::<D>(kind.as_ref())),
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

#[derive(Copy, Clone, Debug)]
struct CredentialPreviewMsgType;

impl From<CredentialPreviewMsgType> for CredentialIssuanceV1_0Kind {
    fn from(_value: CredentialPreviewMsgType) -> Self {
        CredentialIssuanceV1_0Kind::CredentialPreview
    }
}

impl Serialize for CredentialPreviewMsgType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let protocol = MessageFamily::from(CredentialIssuanceV1_0Kind::parent());
        format_args!("{protocol}/{}", CredentialIssuanceV1_0Kind::CredentialPreview.as_ref()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CredentialPreviewMsgType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let msg_type = MessageType::deserialize(deserializer)?;

        if let MessageFamily::CredentialIssuance(CredentialIssuanceKind::V1(CredentialIssuanceV1::V1_0(_))) =
            msg_type.protocol
        {
            if let Ok(CredentialIssuanceV1_0Kind::CredentialPreview) =
                CredentialIssuanceV1_0Kind::from_str(msg_type.kind)
            {
                return Ok(CredentialPreviewMsgType);
            }
        }

        let kind = CredentialIssuanceV1_0Kind::CredentialPreview;
        Err(D::Error::custom(format!("message kind is not {}", kind.as_ref())))
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
