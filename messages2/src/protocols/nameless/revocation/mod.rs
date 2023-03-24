//! Module containing the `revocation notification` protocol messages, as defined in the [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/features/0721-revocation-notification-v2/README.md).

pub mod ack;
pub mod revoke;

use derive_more::From;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use self::{
    ack::{AckRevoke, AckRevokeContent},
    revoke::{Revoke, RevokeContent, RevokeDecorators},
};
use super::notification::AckDecorators;
use crate::{
    misc::utils::{into_msg_with_type, transit_to_aries_msg},
    msg_types::{
        types::revocation::{RevocationProtocol as RevocationKind, RevocationProtocolV2, RevocationProtocolV2_0},
        MsgWithType,
    },
    protocols::traits::DelayedSerde,
};

#[derive(Clone, Debug, From, PartialEq)]
pub enum Revocation {
    Revoke(Revoke),
    Ack(AckRevoke),
}

impl DelayedSerde for Revocation {
    type MsgType<'a> = (RevocationKind, &'a str);

    fn delayed_deserialize<'de, D>(msg_type: Self::MsgType<'de>, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (protocol, kind_str) = msg_type;

        let kind = match protocol {
            RevocationKind::V2(RevocationProtocolV2::V2_0(kind)) => kind.kind_from_str(kind_str),
        };

        match kind.map_err(D::Error::custom)? {
            RevocationProtocolV2_0::Revoke => Revoke::deserialize(deserializer).map(From::from),
            RevocationProtocolV2_0::Ack => AckRevoke::deserialize(deserializer).map(From::from),
        }
    }

    fn delayed_serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Revoke(v) => MsgWithType::from(v).serialize(serializer),
            Self::Ack(v) => MsgWithType::from(v).serialize(serializer),
        }
    }
}

transit_to_aries_msg!(RevokeContent: RevokeDecorators, Revocation);
transit_to_aries_msg!(AckRevokeContent: AckDecorators, Revocation);

into_msg_with_type!(Revoke, RevocationProtocolV2_0, Revoke);
into_msg_with_type!(AckRevoke, RevocationProtocolV2_0, Ack);
