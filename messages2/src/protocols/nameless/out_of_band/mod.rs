//! Module containing the `out of band` protocol messages, as defined in the [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/features/0434-outofband/README.md).

pub mod invitation;
pub mod reuse;
pub mod reuse_accepted;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use self::{
    invitation::{Invitation, InvitationContent, InvitationDecorators},
    reuse::{HandshakeReuse, HandshakeReuseContent, HandshakeReuseDecorators},
    reuse_accepted::{HandshakeReuseAccepted, HandshakeReuseAcceptedContent, HandshakeReuseAcceptedDecorators},
};
use crate::{
    misc::utils::{into_msg_with_type, transit_to_aries_msg},
    msg_types::{
        types::out_of_band::{OutOfBandProtocol as OutOfBandKind, OutOfBandProtocolV1, OutOfBandProtocolV1_1},
        MsgWithType,
    },
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
        let (major, kind_str) = msg_type;

        let kind = match major {
            OutOfBandKind::V1(OutOfBandProtocolV1::V1_1(kind)) => kind.kind_from_str(kind_str),
        };

        match kind.map_err(D::Error::custom)? {
            OutOfBandProtocolV1_1::Invitation => Invitation::deserialize(deserializer).map(From::from),
            OutOfBandProtocolV1_1::HandshakeReuse => HandshakeReuse::deserialize(deserializer).map(From::from),
            OutOfBandProtocolV1_1::HandshakeReuseAccepted => {
                HandshakeReuseAccepted::deserialize(deserializer).map(From::from)
            }
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Invitation(v) => MsgWithType::from(v).serialize(serializer),
            Self::HandshakeReuse(v) => MsgWithType::from(v).serialize(serializer),
            Self::HandshakeReuseAccepted(v) => MsgWithType::from(v).serialize(serializer),
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

into_msg_with_type!(Invitation, OutOfBandProtocolV1_1, Invitation);
into_msg_with_type!(HandshakeReuse, OutOfBandProtocolV1_1, HandshakeReuse);
into_msg_with_type!(HandshakeReuseAccepted, OutOfBandProtocolV1_1, HandshakeReuseAccepted);
