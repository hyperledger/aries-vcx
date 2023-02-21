mod ack;
mod issue_credential;
mod offer_credential;
mod propose_credential;
mod request_credential;

use derive_more::From;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    delayed_serde::DelayedSerde,
    message_type::message_family::cred_issuance::{
        CredentialIssuance as CredentialIssuanceKind, CredentialIssuanceV1, CredentialIssuanceV1_0,
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
pub struct CredentialPreviewData {
    #[serde(rename = "@type")]
    pub _type: String,
    pub attributes: Vec<CredentialValue>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CredentialValue {
    pub name: String,
    pub value: String,
    #[serde(rename = "mime-type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _type: Option<MimeType>,
}
