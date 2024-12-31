//! Module containing the `issue credential` protocol messages, as defined in the [RFC](<https://github.com/hyperledger/aries-rfcs/blob/main/features/0036-issue-credential/README.md>).

pub mod ack;
pub mod issue_credential;
pub mod offer_credential;
pub mod problem_report;
pub mod propose_credential;
pub mod request_credential;

use std::str::FromStr;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use shared::misc::utils::CowStr;

use self::{
    ack::{AckCredentialV1, AckCredentialV1Content},
    issue_credential::{IssueCredentialV1, IssueCredentialV1Content, IssueCredentialV1Decorators},
    offer_credential::{OfferCredentialV1, OfferCredentialV1Content, OfferCredentialV1Decorators},
    problem_report::{CredIssuanceV1ProblemReport, CredIssuanceV1ProblemReportContent},
    propose_credential::{
        ProposeCredentialV1, ProposeCredentialV1Content, ProposeCredentialV1Decorators,
    },
    request_credential::{
        RequestCredentialV1, RequestCredentialV1Content, RequestCredentialV1Decorators,
    },
};
use super::{common::CredentialAttr, CredentialIssuance};
use crate::{
    misc::utils::{self, into_msg_with_type, transit_to_aries_msg},
    msg_fields::{
        protocols::{notification::ack::AckDecorators, report_problem::ProblemReportDecorators},
        traits::DelayedSerde,
    },
    msg_types::{
        protocols::cred_issuance::{
            CredentialIssuanceType as CredentialIssuanceKind, CredentialIssuanceTypeV1,
            CredentialIssuanceTypeV1_0,
        },
        traits::MessageKind,
        MessageType, MsgWithType, Protocol,
    },
};

#[derive(Clone, Debug, From, PartialEq)]
pub enum CredentialIssuanceV1 {
    OfferCredential(OfferCredentialV1),
    ProposeCredential(ProposeCredentialV1),
    RequestCredential(RequestCredentialV1),
    IssueCredential(IssueCredentialV1),
    Ack(AckCredentialV1),
    ProblemReport(CredIssuanceV1ProblemReport),
}

impl DelayedSerde for CredentialIssuanceV1 {
    type MsgType<'a> = (CredentialIssuanceKind, &'a str);

    fn delayed_deserialize<'de, D>(
        msg_type: Self::MsgType<'de>,
        deserializer: D,
    ) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (protocol, kind_str) = msg_type;
        let kind =
            match protocol {
                CredentialIssuanceKind::V1(CredentialIssuanceTypeV1::V1_0(kind)) => {
                    kind.kind_from_str(kind_str)
                }
                CredentialIssuanceKind::V2(_) => return Err(D::Error::custom(
                    "Cannot deserialize issue-credential-v2 message type into issue-credential-v1",
                )),
            };

        match kind.map_err(D::Error::custom)? {
            CredentialIssuanceTypeV1_0::OfferCredential => {
                OfferCredentialV1::deserialize(deserializer).map(From::from)
            }
            CredentialIssuanceTypeV1_0::ProposeCredential => {
                ProposeCredentialV1::deserialize(deserializer).map(From::from)
            }
            CredentialIssuanceTypeV1_0::RequestCredential => {
                RequestCredentialV1::deserialize(deserializer).map(From::from)
            }
            CredentialIssuanceTypeV1_0::IssueCredential => {
                IssueCredentialV1::deserialize(deserializer).map(From::from)
            }
            CredentialIssuanceTypeV1_0::Ack => {
                AckCredentialV1::deserialize(deserializer).map(From::from)
            }
            CredentialIssuanceTypeV1_0::ProblemReport => {
                CredIssuanceV1ProblemReport::deserialize(deserializer).map(From::from)
            }
            CredentialIssuanceTypeV1_0::CredentialPreview => {
                Err(utils::not_standalone_msg::<D>(kind_str))
            }
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
            Self::ProblemReport(v) => MsgWithType::from(v).serialize(serializer),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CredentialPreviewV1 {
    #[serde(rename = "@type")]
    msg_type: CredentialPreviewV1MsgType,
    pub attributes: Vec<CredentialAttr>,
}

impl CredentialPreviewV1 {
    pub fn new(attributes: Vec<CredentialAttr>) -> Self {
        Self {
            msg_type: CredentialPreviewV1MsgType,
            attributes,
        }
    }
}

/// Non-standalone message type.
/// This is only encountered as part of an existent message.
/// It is not a message on it's own.
#[derive(Copy, Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(try_from = "CowStr")]
struct CredentialPreviewV1MsgType;

impl<'a> From<&'a CredentialPreviewV1MsgType> for CredentialIssuanceTypeV1_0 {
    fn from(_value: &'a CredentialPreviewV1MsgType) -> Self {
        CredentialIssuanceTypeV1_0::CredentialPreview
    }
}

impl TryFrom<CowStr<'_>> for CredentialPreviewV1MsgType {
    type Error = String;

    fn try_from(value: CowStr) -> Result<Self, Self::Error> {
        let value = MessageType::try_from(value.0.as_ref())?;

        if let Protocol::CredentialIssuanceType(CredentialIssuanceKind::V1(
            CredentialIssuanceTypeV1::V1_0(_),
        )) = value.protocol
        {
            if let Ok(CredentialIssuanceTypeV1_0::CredentialPreview) =
                CredentialIssuanceTypeV1_0::from_str(value.kind)
            {
                return Ok(CredentialPreviewV1MsgType);
            }
        }

        Err(format!("message kind is not {}", value.kind))
    }
}

impl Serialize for CredentialPreviewV1MsgType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let protocol = Protocol::from(CredentialIssuanceTypeV1_0::parent());
        let kind = CredentialIssuanceTypeV1_0::from(self);
        format_args!("{protocol}/{}", kind.as_ref()).serialize(serializer)
    }
}

transit_to_aries_msg!(
    OfferCredentialV1Content: OfferCredentialV1Decorators,
    CredentialIssuanceV1, CredentialIssuance
);
transit_to_aries_msg!(
    ProposeCredentialV1Content: ProposeCredentialV1Decorators,
    CredentialIssuanceV1, CredentialIssuance
);
transit_to_aries_msg!(
    RequestCredentialV1Content: RequestCredentialV1Decorators,
    CredentialIssuanceV1, CredentialIssuance
);
transit_to_aries_msg!(
    IssueCredentialV1Content: IssueCredentialV1Decorators,
    CredentialIssuanceV1, CredentialIssuance
);
transit_to_aries_msg!(AckCredentialV1Content: AckDecorators, CredentialIssuanceV1, CredentialIssuance);
transit_to_aries_msg!(
    CredIssuanceV1ProblemReportContent: ProblemReportDecorators,
    CredentialIssuanceV1, CredentialIssuance
);

into_msg_with_type!(
    OfferCredentialV1,
    CredentialIssuanceTypeV1_0,
    OfferCredential
);
into_msg_with_type!(
    ProposeCredentialV1,
    CredentialIssuanceTypeV1_0,
    ProposeCredential
);
into_msg_with_type!(
    RequestCredentialV1,
    CredentialIssuanceTypeV1_0,
    RequestCredential
);
into_msg_with_type!(
    IssueCredentialV1,
    CredentialIssuanceTypeV1_0,
    IssueCredential
);
into_msg_with_type!(AckCredentialV1, CredentialIssuanceTypeV1_0, Ack);
into_msg_with_type!(
    CredIssuanceV1ProblemReport,
    CredentialIssuanceTypeV1_0,
    ProblemReport
);
