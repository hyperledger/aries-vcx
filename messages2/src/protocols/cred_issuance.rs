use derive_more::From;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{message_type::message_family::{
    cred_issuance::{CredentialIssuance as CredentialIssuanceKind, CredentialIssuanceV1, CredentialIssuanceV1_0},
}, delayed_serde::DelayedSerde};

use super::traits::ConcreteMessage;

#[derive(Clone, Debug, From)]
pub enum CredentialIssuance {
    OfferCredential(OfferCredential),
    ProposeCredential(ProposeCredential),
    RequestCredential(RequestCredential),
    IssueCredential(IssueCredential),
    Ack(Ack),
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
            CredentialIssuanceV1_0::Ack => Ack::deserialize(deserializer).map(From::from),
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OfferCredential;

impl ConcreteMessage for OfferCredential {
    type Kind = CredentialIssuanceV1_0;

    fn kind() -> Self::Kind {
        Self::Kind::OfferCredential
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProposeCredential;

impl ConcreteMessage for ProposeCredential {
    type Kind = CredentialIssuanceV1_0;

    fn kind() -> Self::Kind {
        Self::Kind::ProposeCredential
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestCredential;

impl ConcreteMessage for RequestCredential {
    type Kind = CredentialIssuanceV1_0;

    fn kind() -> Self::Kind {
        Self::Kind::RequestCredential
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IssueCredential;

impl ConcreteMessage for IssueCredential {
    type Kind = CredentialIssuanceV1_0;

    fn kind() -> Self::Kind {
        Self::Kind::IssueCredential
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ack;

impl ConcreteMessage for Ack {
    type Kind = CredentialIssuanceV1_0;

    fn kind() -> Self::Kind {
        Self::Kind::Ack
    }
}
