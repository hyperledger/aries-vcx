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
        utils::{self, into_msg_with_type, transit_to_aries_msg},
        MimeType,
    },
    msg_types::{
        traits::{MessageKind, ProtocolVersion},
        types::cred_issuance::{
            CredentialIssuanceProtocol as CredentialIssuanceKind, CredentialIssuanceProtocolV1,
            CredentialIssuanceProtocolV1_0,
        },
        MessageType, Protocol, MsgWithType,
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
        let (major, kind_str) = msg_type;
        let kind = match major {
            CredentialIssuanceKind::V1(CredentialIssuanceProtocolV1::V1_0(pd)) => {
                CredentialIssuanceProtocolV1::kind(pd, kind_str)
            }
        };

        match kind.map_err(D::Error::custom)? {
            CredentialIssuanceProtocolV1_0::OfferCredential => {
                OfferCredential::deserialize(deserializer).map(From::from)
            }
            CredentialIssuanceProtocolV1_0::ProposeCredential => {
                ProposeCredential::deserialize(deserializer).map(From::from)
            }
            CredentialIssuanceProtocolV1_0::RequestCredential => {
                RequestCredential::deserialize(deserializer).map(From::from)
            }
            CredentialIssuanceProtocolV1_0::IssueCredential => {
                IssueCredential::deserialize(deserializer).map(From::from)
            }
            CredentialIssuanceProtocolV1_0::Ack => AckCredential::deserialize(deserializer).map(From::from),
            CredentialIssuanceProtocolV1_0::CredentialPreview => Err(utils::not_standalone_msg::<D>(kind_str)),
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::OfferCredential(v) => MsgWithType::from(v).serialize(serializer),
            Self::ProposeCredential(v) => MsgWithType::from(v).serialize(serializer),
            Self::RequestCredential(v) => MsgWithType::from(v).serialize(serializer),
            Self::IssueCredential(v) => MsgWithType::from(v).serialize(serializer),
            Self::Ack(v) => MsgWithType::from(v).serialize(serializer),
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

impl<'a> From<&'a CredentialPreviewMsgType> for CredentialIssuanceProtocolV1_0 {
    fn from(_value: &'a CredentialPreviewMsgType) -> Self {
        CredentialIssuanceProtocolV1_0::CredentialPreview
    }
}

impl<'a> TryFrom<MessageType<'a>> for CredentialPreviewMsgType {
    type Error = String;

    fn try_from(value: MessageType) -> Result<Self, Self::Error> {
        if let Protocol::CredentialIssuanceProtocol(CredentialIssuanceKind::V1(CredentialIssuanceProtocolV1::V1_0(_))) =
            value.protocol
        {
            if let Ok(CredentialIssuanceProtocolV1_0::CredentialPreview) =
                CredentialIssuanceProtocolV1_0::from_str(value.kind)
            {
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
        let protocol = Protocol::from(CredentialIssuanceProtocolV1_0::parent());
        let kind = CredentialIssuanceProtocolV1_0::from(self);
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

into_msg_with_type!(OfferCredential, CredentialIssuanceProtocolV1_0, OfferCredential);
into_msg_with_type!(ProposeCredential, CredentialIssuanceProtocolV1_0, ProposeCredential);
into_msg_with_type!(RequestCredential, CredentialIssuanceProtocolV1_0, RequestCredential);
into_msg_with_type!(IssueCredential, CredentialIssuanceProtocolV1_0, IssueCredential);
into_msg_with_type!(AckCredential, CredentialIssuanceProtocolV1_0, Ack);
