mod invitation;
mod reuse;
mod reuse_accepted;

use std::str::FromStr;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

pub use self::{invitation::Invitation, reuse::HandshakeReuse, reuse_accepted::HandshakeReuseAccepted};
use self::{
    invitation::{InvitationContent, InvitationDecorators},
    reuse::{HandshakeReuseContent, HandshakeReuseDecorators},
    reuse_accepted::{HandshakeReuseAcceptedContent, HandshakeReuseAcceptedDecorators},
};
use crate::{
    misc::utils::transit_to_aries_msg,
    msg_types::types::out_of_band::{OutOfBand as OutOfBandKind, OutOfBandV1, OutOfBandV1_1Kind},
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
        let kind = OutOfBandV1_1Kind::from_str(kind).map_err(D::Error::custom)?;

        match kind {
            OutOfBandV1_1Kind::Invitation => Invitation::delayed_deserialize(kind, deserializer).map(From::from),
            OutOfBandV1_1Kind::HandshakeReuse => {
                HandshakeReuse::delayed_deserialize(kind, deserializer).map(From::from)
            }
            OutOfBandV1_1Kind::HandshakeReuseAccepted => {
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
