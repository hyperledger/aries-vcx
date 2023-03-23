//! Module containing the `out of band` protocol messages, as defined in the [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/features/0434-outofband/README.md).

pub mod invitation;
pub mod reuse;
pub mod reuse_accepted;

use std::str::FromStr;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use self::{
    invitation::{Invitation, InvitationContent, InvitationDecorators},
    reuse::{HandshakeReuse, HandshakeReuseContent, HandshakeReuseDecorators},
    reuse_accepted::{HandshakeReuseAccepted, HandshakeReuseAcceptedContent, HandshakeReuseAcceptedDecorators},
};
use crate::{
    misc::utils::transit_to_aries_msg,
    msg_types::types::out_of_band::{OutOfBand as OutOfBandKind, OutOfBandV1, OutOfBandV1_1},
    protocols::traits::DelayedSerde,
};

#[derive(Clone, Debug, From, PartialEq)]
pub enum OutOfBand {
    Invitation(Invitation),
    HandshakeReuse(HandshakeReuse),
    HandshakeReuseAccepted(HandshakeReuseAccepted),
}

impl DelayedSerde for OutOfBand {
    type MsgType<'a> = (OutOfBandKind, &'a str);

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (major, kind) = msg_type;
        let OutOfBandKind::V1(major) = major;
        let OutOfBandV1::V1_1(_minor) = major;
        let kind = OutOfBandV1_1::from_str(kind).map_err(D::Error::custom)?;

        match kind {
            OutOfBandV1_1::Invitation => Invitation::delayed_deserialize(kind, deserializer).map(From::from),
            OutOfBandV1_1::HandshakeReuse => HandshakeReuse::delayed_deserialize(kind, deserializer).map(From::from),
            OutOfBandV1_1::HandshakeReuseAccepted => {
                HandshakeReuseAccepted::delayed_deserialize(kind, deserializer).map(From::from)
            }
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Invitation(v) => v.delayed_serialize(serializer),
            Self::HandshakeReuse(v) => v.delayed_serialize(serializer),
            Self::HandshakeReuseAccepted(v) => v.delayed_serialize(serializer),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum OobGoalCode {
    #[serde(rename = "issue-vc")]
    IssueVC,
    #[serde(rename = "request-proof")]
    RequestProof,
    #[serde(rename = "create-account")]
    CreateAccount,
    #[serde(rename = "p2p-messaging")]
    P2PMessaging,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum HandshakeProtocol {
    ConnectionV1,
    DidExchangeV1,
}

transit_to_aries_msg!(InvitationContent: InvitationDecorators, OutOfBand);
transit_to_aries_msg!(HandshakeReuseContent: HandshakeReuseDecorators, OutOfBand);
transit_to_aries_msg!(
    HandshakeReuseAcceptedContent: HandshakeReuseAcceptedDecorators,
    OutOfBand
);
