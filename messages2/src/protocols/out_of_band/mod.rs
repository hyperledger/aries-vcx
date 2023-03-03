mod invitation;
mod reuse;
mod reuse_accepted;

use derive_more::From;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    composite_message::Message,
    delayed_serde::DelayedSerde,
    message_type::message_family::out_of_band::{OutOfBand as OutOfBandKind, OutOfBandV1, OutOfBandV1_1},
};

use self::{
    invitation::{Invitation, InvitationDecorators},
    reuse::{HandshakeReuse, HandshakeReuseDecorators},
    reuse_accepted::{HandshakeReuseAccepted, HandshakeReuseAcceptedDecorators},
};

#[derive(Clone, Debug, From)]
pub enum OutOfBand {
    Invitation(Message<Invitation, InvitationDecorators>),
    HandshakeReuse(Message<HandshakeReuse, HandshakeReuseDecorators>),
    HandshakeReuseAccepted(Message<HandshakeReuseAccepted, HandshakeReuseAcceptedDecorators>),
}

impl DelayedSerde for OutOfBand {
    type MsgType = OutOfBandKind;

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let OutOfBandKind::V1(major) = msg_type;
        let OutOfBandV1::V1_1(minor) = major;

        match minor {
            OutOfBandV1_1::Invitation => {
                Message::<Invitation, InvitationDecorators>::delayed_deserialize(minor, deserializer).map(From::from)
            }
            OutOfBandV1_1::HandshakeReuse => {
                Message::<HandshakeReuse, HandshakeReuseDecorators>::delayed_deserialize(minor, deserializer)
                    .map(From::from)
            }
            OutOfBandV1_1::HandshakeReuseAccepted => {
                Message::<HandshakeReuseAccepted, HandshakeReuseAcceptedDecorators>::delayed_deserialize(
                    minor,
                    deserializer,
                )
                .map(From::from)
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
