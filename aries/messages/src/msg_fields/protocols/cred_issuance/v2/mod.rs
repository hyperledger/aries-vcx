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
    ack::{AckCredentialV2, AckCredentialV2Content},
    issue_credential::{IssueCredentialV2, IssueCredentialV2Content, IssueCredentialV2Decorators},
    offer_credential::{OfferCredentialV2, OfferCredentialV2Content, OfferCredentialV2Decorators},
    problem_report::{CredIssuanceProblemReportV2, CredIssuanceV2ProblemReportContent},
    propose_credential::{
        ProposeCredentialV2, ProposeCredentialV2Content, ProposeCredentialV2Decorators,
    },
    request_credential::{
        RequestCredentialV2, RequestCredentialV2Content, RequestCredentialV2Decorators,
    },
};
use super::{
    super::{notification::ack::AckDecorators, report_problem::ProblemReportDecorators},
    common::CredentialAttr,
    CredentialIssuance,
};
use crate::{
    misc::utils::{self, into_msg_with_type, transit_to_aries_msg},
    msg_fields::traits::DelayedSerde,
    msg_types::{
        cred_issuance::{CredentialIssuanceTypeV2, CredentialIssuanceTypeV2_0},
        protocols::cred_issuance::CredentialIssuanceType as CredentialIssuanceKind,
        traits::MessageKind,
        MessageType, MsgWithType, Protocol,
    },
};

#[derive(Clone, Debug, From, PartialEq)]
pub enum CredentialIssuanceV2 {
    OfferCredential(OfferCredentialV2),
    ProposeCredential(ProposeCredentialV2),
    RequestCredential(RequestCredentialV2),
    IssueCredential(IssueCredentialV2),
    Ack(AckCredentialV2),
    ProblemReport(CredIssuanceProblemReportV2),
}

impl DelayedSerde for CredentialIssuanceV2 {
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
                CredentialIssuanceKind::V2(CredentialIssuanceTypeV2::V2_0(kind)) => {
                    kind.kind_from_str(kind_str)
                }
                CredentialIssuanceKind::V1(_) => return Err(D::Error::custom(
                    "Cannot deserialize issue-credential-v1 message type into issue-credential-v2",
                )),
            };

        match kind.map_err(D::Error::custom)? {
            CredentialIssuanceTypeV2_0::OfferCredential => {
                OfferCredentialV2::deserialize(deserializer).map(From::from)
            }
            CredentialIssuanceTypeV2_0::ProposeCredential => {
                ProposeCredentialV2::deserialize(deserializer).map(From::from)
            }
            CredentialIssuanceTypeV2_0::RequestCredential => {
                RequestCredentialV2::deserialize(deserializer).map(From::from)
            }
            CredentialIssuanceTypeV2_0::IssueCredential => {
                IssueCredentialV2::deserialize(deserializer).map(From::from)
            }
            CredentialIssuanceTypeV2_0::Ack => {
                AckCredentialV2::deserialize(deserializer).map(From::from)
            }
            CredentialIssuanceTypeV2_0::ProblemReport => {
                CredIssuanceProblemReportV2::deserialize(deserializer).map(From::from)
            }
            CredentialIssuanceTypeV2_0::CredentialPreview => {
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
pub struct CredentialPreviewV2 {
    #[serde(rename = "@type")]
    msg_type: CredentialPreviewV2MsgType,
    pub attributes: Vec<CredentialAttr>,
}

impl CredentialPreviewV2 {
    pub fn new(attributes: Vec<CredentialAttr>) -> Self {
        Self {
            msg_type: CredentialPreviewV2MsgType,
            attributes,
        }
    }
}

/// Non-standalone message type.
/// This is only encountered as part of an existent message.
/// It is not a message on it's own.
#[derive(Copy, Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(try_from = "CowStr")]
struct CredentialPreviewV2MsgType;

impl<'a> From<&'a CredentialPreviewV2MsgType> for CredentialIssuanceTypeV2_0 {
    fn from(_value: &'a CredentialPreviewV2MsgType) -> Self {
        CredentialIssuanceTypeV2_0::CredentialPreview
    }
}

impl TryFrom<CowStr<'_>> for CredentialPreviewV2MsgType {
    type Error = String;

    fn try_from(value: CowStr) -> Result<Self, Self::Error> {
        let value = MessageType::try_from(value.0.as_ref())?;

        if let Protocol::CredentialIssuanceType(CredentialIssuanceKind::V2(
            CredentialIssuanceTypeV2::V2_0(_),
        )) = value.protocol
        {
            if let Ok(CredentialIssuanceTypeV2_0::CredentialPreview) =
                CredentialIssuanceTypeV2_0::from_str(value.kind)
            {
                return Ok(CredentialPreviewV2MsgType);
            }
        }

        Err(format!("message kind is not {}", value.kind))
    }
}

impl Serialize for CredentialPreviewV2MsgType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let protocol = Protocol::from(CredentialIssuanceTypeV2_0::parent());
        let kind = CredentialIssuanceTypeV2_0::from(self);
        format_args!("{protocol}/{}", kind.as_ref()).serialize(serializer)
    }
}

transit_to_aries_msg!(
    OfferCredentialV2Content: OfferCredentialV2Decorators,
    CredentialIssuanceV2, CredentialIssuance
);
transit_to_aries_msg!(
    ProposeCredentialV2Content: ProposeCredentialV2Decorators,
    CredentialIssuanceV2, CredentialIssuance
);
transit_to_aries_msg!(
    RequestCredentialV2Content: RequestCredentialV2Decorators,
    CredentialIssuanceV2, CredentialIssuance
);
transit_to_aries_msg!(
    IssueCredentialV2Content: IssueCredentialV2Decorators,
    CredentialIssuanceV2, CredentialIssuance
);
transit_to_aries_msg!(AckCredentialV2Content: AckDecorators, CredentialIssuanceV2, CredentialIssuance);
transit_to_aries_msg!(
    CredIssuanceV2ProblemReportContent: ProblemReportDecorators,
    CredentialIssuanceV2, CredentialIssuance
);

into_msg_with_type!(
    OfferCredentialV2,
    CredentialIssuanceTypeV2_0,
    OfferCredential
);
into_msg_with_type!(
    ProposeCredentialV2,
    CredentialIssuanceTypeV2_0,
    ProposeCredential
);
into_msg_with_type!(
    RequestCredentialV2,
    CredentialIssuanceTypeV2_0,
    RequestCredential
);
into_msg_with_type!(
    IssueCredentialV2,
    CredentialIssuanceTypeV2_0,
    IssueCredential
);
into_msg_with_type!(AckCredentialV2, CredentialIssuanceTypeV2_0, Ack);
into_msg_with_type!(
    CredIssuanceProblemReportV2,
    CredentialIssuanceTypeV2_0,
    ProblemReport
);
