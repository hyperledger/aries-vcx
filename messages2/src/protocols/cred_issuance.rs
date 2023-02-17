use derive_more::From;
use messages_macros::Message;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    delayed_serde::DelayedSerde,
    message_type::message_family::cred_issuance::{
        CredentialIssuance as CredentialIssuanceKind, CredentialIssuanceV1, CredentialIssuanceV1_0,
    },
};

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

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "CredentialIssuanceV1_0::OfferCredential")]
pub struct OfferCredential;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "CredentialIssuanceV1_0::ProposeCredential")]
pub struct ProposeCredential;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "CredentialIssuanceV1_0::RequestCredential")]
pub struct RequestCredential;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "CredentialIssuanceV1_0::IssueCredential")]
pub struct IssueCredential;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "CredentialIssuanceV1_0::Ack")]
pub struct Ack;
